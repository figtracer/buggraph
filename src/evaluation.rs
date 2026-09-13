//! Reproducible retrieval diagnostics with explicit labels and split hygiene.

use crate::{Graph, Kind, RetrievalMode, TokenCounter, retrieval::terms};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalSuite {
    pub revision: String,
    pub corpus_revision: String,
    pub description: String,
    pub cases: Vec<EvalCase>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Split {
    Dev,
    Test,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalCase {
    pub id: String,
    pub group: String,
    pub split: Split,
    pub query: String,
    #[serde(default)]
    pub facets: Vec<String>,
    pub relevant_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CaseResult {
    pub id: String,
    pub split: Split,
    pub retrieved: Vec<String>,
    pub relevant: usize,
    pub found: usize,
    pub tokens: usize,
    pub precision: f64,
    pub recall: f64,
    pub reciprocal_rank: f64,
    pub ndcg: f64,
}

#[derive(Debug, Serialize)]
pub struct Metrics {
    pub split: Split,
    pub positive_cases: usize,
    pub negative_cases: usize,
    pub precision_at_k: Option<f64>,
    pub recall_at_k: Option<f64>,
    pub mrr_at_k: Option<f64>,
    pub ndcg_at_k: Option<f64>,
    pub negative_empty_rate: Option<f64>,
    pub mean_context_tokens: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ModeResult {
    pub mode: RetrievalMode,
    pub metrics: Vec<Metrics>,
    pub cases: Vec<CaseResult>,
}

#[derive(Debug, Serialize)]
pub struct EvalReport {
    pub suite_revision: String,
    pub corpus_revision: String,
    pub model: String,
    pub max_tokens: usize,
    pub k: usize,
    pub results: Vec<ModeResult>,
}

impl Graph {
    pub fn evaluate(
        &self,
        suite: &EvalSuite,
        counter: &TokenCounter,
        max_tokens: usize,
        k: usize,
    ) -> Result<EvalReport, String> {
        if suite.corpus_revision != self.corpus.revision
            || suite.revision.trim().is_empty()
            || suite.description.trim().is_empty()
            || suite.cases.is_empty()
            || k == 0
        {
            return Err(
                "evaluation requires matching revisions, description, cases and positive k".into(),
            );
        }
        let mut ids = HashSet::new();
        let mut queries = HashSet::new();
        let mut groups = HashMap::new();
        for case in &suite.cases {
            if case.id.trim().is_empty()
                || case.group.trim().is_empty()
                || terms(&case.query).is_empty()
                || !ids.insert(&case.id)
                || !queries.insert(terms(&case.query))
            {
                return Err(
                    "evaluation case IDs and normalized queries must be nonempty and unique".into(),
                );
            }
            if groups
                .insert(&case.group, case.split)
                .is_some_and(|previous| previous != case.split)
            {
                return Err(format!("group appears across splits: {}", case.group));
            }
            let relevant = case.relevant_ids.iter().collect::<HashSet<_>>();
            if relevant.len() != case.relevant_ids.len()
                || relevant.iter().any(|id| {
                    self.node(id)
                        .is_none_or(|node| node.kind != Kind::FailureMode)
                })
            {
                return Err(format!(
                    "invalid or duplicate relevance labels: {}",
                    case.id
                ));
            }
            for facet in &case.facets {
                if !self.facets.contains_key(facet) {
                    return Err(format!("unknown evaluation facet: {facet}"));
                }
            }
            if relevant.iter().any(|id| {
                case.facets
                    .iter()
                    .any(|facet| !self.node(id).unwrap().facets.contains(facet))
            }) {
                return Err(format!(
                    "relevance labels contradict facet filter: {}",
                    case.id
                ));
            }
        }
        let mut results = Vec::new();
        for mode in [
            RetrievalMode::IdOrder,
            RetrievalMode::Bm25,
            RetrievalMode::Bm25Ancestors,
        ] {
            let mut cases = Vec::new();
            for case in &suite.cases {
                let facets = case.facets.iter().map(String::as_str).collect::<Vec<_>>();
                let context =
                    self.ranked_context(&case.query, &facets, mode, counter, max_tokens, k);
                let relevant = case
                    .relevant_ids
                    .iter()
                    .map(String::as_str)
                    .collect::<HashSet<_>>();
                let positions = context
                    .hits
                    .iter()
                    .enumerate()
                    .filter_map(|(i, hit)| relevant.contains(hit.id).then_some(i + 1))
                    .collect::<Vec<_>>();
                let dcg = positions
                    .iter()
                    .map(|&position| 1.0 / ((position + 1) as f64).log2())
                    .sum::<f64>();
                let ideal = (1..=k.min(relevant.len()))
                    .map(|position| 1.0 / ((position + 1) as f64).log2())
                    .sum::<f64>();
                cases.push(CaseResult {
                    id: case.id.clone(),
                    split: case.split,
                    retrieved: context.hits.iter().map(|hit| hit.id.to_owned()).collect(),
                    relevant: relevant.len(),
                    found: positions.len(),
                    tokens: context.tokens,
                    precision: positions.len() as f64 / k as f64,
                    recall: if relevant.is_empty() {
                        0.0
                    } else {
                        positions.len() as f64 / relevant.len() as f64
                    },
                    reciprocal_rank: positions
                        .first()
                        .map_or(0.0, |&position| 1.0 / position as f64),
                    ndcg: if ideal == 0.0 { 0.0 } else { dcg / ideal },
                });
            }
            let metrics = [Split::Dev, Split::Test]
                .into_iter()
                .map(|split| {
                    let rows = cases
                        .iter()
                        .filter(|case| case.split == split)
                        .collect::<Vec<_>>();
                    let positive = rows
                        .iter()
                        .copied()
                        .filter(|case| case.relevant > 0)
                        .collect::<Vec<_>>();
                    let negative = rows
                        .iter()
                        .copied()
                        .filter(|case| case.relevant == 0)
                        .collect::<Vec<_>>();
                    let mean = |sum: f64, count: usize| (count > 0).then(|| sum / count as f64);
                    Metrics {
                        split,
                        positive_cases: positive.len(),
                        negative_cases: negative.len(),
                        precision_at_k: mean(
                            positive.iter().map(|case| case.precision).sum(),
                            positive.len(),
                        ),
                        recall_at_k: mean(
                            positive.iter().map(|case| case.recall).sum(),
                            positive.len(),
                        ),
                        mrr_at_k: mean(
                            positive.iter().map(|case| case.reciprocal_rank).sum(),
                            positive.len(),
                        ),
                        ndcg_at_k: mean(
                            positive.iter().map(|case| case.ndcg).sum(),
                            positive.len(),
                        ),
                        negative_empty_rate: mean(
                            negative
                                .iter()
                                .filter(|case| case.retrieved.is_empty())
                                .count() as f64,
                            negative.len(),
                        ),
                        mean_context_tokens: mean(
                            rows.iter().map(|case| case.tokens as f64).sum(),
                            rows.len(),
                        ),
                    }
                })
                .collect();
            results.push(ModeResult {
                mode,
                metrics,
                cases,
            });
        }
        Ok(EvalReport {
            suite_revision: suite.revision.clone(),
            corpus_revision: self.corpus.revision.clone(),
            model: counter.model().into(),
            max_tokens,
            k,
            results,
        })
    }
}
