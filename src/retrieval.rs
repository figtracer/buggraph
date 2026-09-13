//! Lexical retrieval and exact model-token budgets for serialized context.

use crate::{Graph, Kind, Node};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use tiktoken_rs::{CoreBPE, bpe_for_model};

/// Conventional BM25 defaults, fixed before evaluation; not fitted to the suite.
const K1: f64 = 1.2;
const B: f64 = 0.75;

pub(crate) fn terms(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}

pub(crate) struct SearchIndex {
    postings: HashMap<String, Vec<(usize, f64)>>,
    lengths: Vec<usize>,
    average: f64,
    documents: usize,
}

impl SearchIndex {
    pub(crate) fn build(nodes: &[Node]) -> Self {
        let mut postings = HashMap::<String, Vec<(usize, f64)>>::new();
        let mut lengths = vec![0; nodes.len()];
        let mut documents = 0;
        for (index, node) in nodes.iter().enumerate() {
            if node.kind == Kind::FailureMode {
                documents += 1;
                let mut frequencies = HashMap::<String, usize>::new();
                // Exclusions and source titles are not positive relevance evidence.
                for field in [&node.summary, &node.definition]
                    .into_iter()
                    .chain(&node.facets)
                    .chain(&node.applicability)
                {
                    for term in terms(field) {
                        lengths[index] += 1;
                        *frequencies.entry(term).or_default() += 1;
                    }
                }
                for (term, frequency) in frequencies {
                    postings
                        .entry(term)
                        .or_default()
                        .push((index, frequency as f64));
                }
            }
        }
        let average = lengths.iter().sum::<usize>() as f64 / documents.max(1) as f64;
        Self {
            postings,
            lengths,
            average: average.max(1.0),
            documents,
        }
    }

    fn scores(&self, query: &str) -> HashMap<usize, f64> {
        let mut scores = HashMap::<usize, f64>::new();
        let mut query_terms = terms(query);
        query_terms.sort_unstable();
        query_terms.dedup();
        for term in query_terms {
            if let Some(postings) = self.postings.get(&term) {
                let idf = (1.0
                    + (self.documents as f64 - postings.len() as f64 + 0.5)
                        / (postings.len() as f64 + 0.5))
                    .ln();
                for &(index, frequency) in postings {
                    let denominator =
                        frequency + K1 * (1.0 - B + B * self.lengths[index] as f64 / self.average);
                    *scores.entry(index).or_default() += idf * frequency * (K1 + 1.0) / denominator;
                }
            }
        }
        scores
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalMode {
    IdOrder,
    Bm25,
    Bm25Ancestors,
}

#[derive(Debug, Serialize)]
pub struct Hit<'a> {
    pub id: &'a str,
    pub score: f64,
    pub relation: &'static str,
}

/// Counts ordinary text, including strings resembling special token delimiters.
/// Counts exclude API message envelopes, tools, instructions, and completion tokens.
pub struct TokenCounter {
    model: String,
    bpe: &'static CoreBPE,
}

impl TokenCounter {
    pub fn for_model(model: &str) -> Result<Self, String> {
        let bpe = bpe_for_model(model).map_err(|error| error.to_string())?;
        Ok(Self {
            model: model.into(),
            bpe,
        })
    }

    pub fn count(&self, text: &str) -> usize {
        self.bpe.encode_ordinary(text).len()
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}

pub struct RankedContext<'a> {
    pub jsonl: String,
    pub hits: Vec<Hit<'a>>,
    pub tokens: usize,
    pub omitted: usize,
}

impl Graph {
    /// Rank only failure modes. Ancestors are explicit structural context and keep
    /// the originating lexical score; they are not independent semantic matches.
    pub fn rank(&self, query: &str, facets: &[&str], mode: RetrievalMode) -> Vec<Hit<'_>> {
        // Unfiltered lexical queries touch only term postings, not every node.
        let candidates =
            (!facets.is_empty()).then(|| self.matching(facets).into_iter().collect::<HashSet<_>>());
        let eligible = |index: usize| {
            self.corpus.nodes[index].kind == Kind::FailureMode
                && candidates.as_ref().is_none_or(|set| set.contains(&index))
        };
        let mut ranked = match mode {
            RetrievalMode::IdOrder => (0..self.corpus.nodes.len())
                .filter(|&i| eligible(i))
                .map(|i| (i, 0.0))
                .collect::<Vec<_>>(),
            _ => self
                .search
                .scores(query)
                .into_iter()
                .filter(|(i, _)| eligible(*i))
                .collect(),
        };
        ranked.sort_unstable_by(|(a, x), (b, y)| y.total_cmp(x).then(a.cmp(b)));
        let mut output = Vec::new();
        let mut seen = HashSet::new();
        for (index, score) in ranked {
            if seen.insert(index) {
                output.push(Hit {
                    id: &self.corpus.nodes[index].id,
                    score,
                    relation: "match",
                });
            }
            if matches!(mode, RetrievalMode::Bm25Ancestors) {
                let mut pending = self.parents[index].iter().copied().collect::<VecDeque<_>>();
                let mut walked = HashSet::new();
                while let Some(parent) = pending.pop_front() {
                    if walked.insert(parent) {
                        if eligible(parent) && seen.insert(parent) {
                            output.push(Hit {
                                id: &self.corpus.nodes[parent].id,
                                score,
                                relation: "ancestor",
                            });
                        }
                        pending.extend(&self.parents[parent]);
                    }
                }
            }
        }
        output
    }

    /// Keep whole records and recount the complete candidate output: BPE token
    /// counts are not assumed additive across record boundaries.
    pub fn ranked_context(
        &self,
        query: &str,
        facets: &[&str],
        mode: RetrievalMode,
        counter: &TokenCounter,
        max_tokens: usize,
        max_records: usize,
    ) -> RankedContext<'_> {
        let ranked = self.rank(query, facets, mode);
        let total = ranked.len();
        let mut result = RankedContext {
            jsonl: String::new(),
            hits: Vec::new(),
            tokens: 0,
            omitted: 0,
        };
        for hit in ranked {
            if result.hits.len() == max_records {
                break;
            }
            let previous = result.jsonl.len();
            result.jsonl.push_str(&self.summaries[self.ids[hit.id]]);
            let tokens = counter.count(&result.jsonl);
            if tokens <= max_tokens {
                result.tokens = tokens;
                result.hits.push(hit);
            } else {
                result.jsonl.truncate(previous);
            }
        }
        result.omitted = total - result.hits.len();
        result
    }
}
