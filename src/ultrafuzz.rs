//! Deterministic routing from an UltraFuzz threat model to failure-mode records.

use crate::{Graph, RetrievalMode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const THREAT_WEIGHT: f64 = 4.0;
const INVARIANT_WEIGHT: f64 = 2.0;
const COVERAGE_GAP_WEIGHT: f64 = 1.0;
const RESULTS_PER_QUERY: usize = 5;
const RRF_OFFSET: usize = 60;

#[derive(Deserialize)]
struct ThreatModel {
    schema_version: String,
    #[serde(default)]
    attack_surfaces: Vec<NamedItem>,
    #[serde(default)]
    invariants: Vec<NamedItem>,
    #[serde(default)]
    threats: Vec<Threat>,
    #[serde(default)]
    coverage_gaps: Vec<NamedItem>,
}

#[derive(Deserialize)]
struct NamedItem {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    statement: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    entry_points: Vec<String>,
}

#[derive(Deserialize)]
struct Threat {
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    impact: String,
    #[serde(default)]
    preconditions: Vec<String>,
    #[serde(default)]
    invariant_ids: Vec<String>,
    #[serde(default)]
    attack_surface_ids: Vec<String>,
}

struct Query {
    id: String,
    weight: f64,
    text: String,
}

/// One lexical vote supporting a routed failure mode.
#[derive(Debug, Serialize)]
pub struct UltraFuzzRouteEvidence {
    pub query_id: String,
    pub rank: usize,
    pub score: f64,
}

/// A failure mode selected through weighted reciprocal-rank fusion.
#[derive(Debug, Serialize)]
pub struct UltraFuzzRouteSelection<'a> {
    pub id: &'a str,
    pub score: f64,
    pub evidence: Vec<UltraFuzzRouteEvidence>,
}

/// A reproducible, locally computed route for one verified UltraFuzz threat model.
#[derive(Debug, Serialize)]
pub struct UltraFuzzRoute<'a> {
    pub schema: &'static str,
    pub corpus_revision: &'a str,
    pub threat_model_sha256: String,
    pub query_count: usize,
    pub max_classes: usize,
    pub selected: Vec<UltraFuzzRouteSelection<'a>>,
}

impl Graph {
    /// Route an UltraFuzz v1 threat model without sending the taxonomy to a model.
    pub fn route_ultrafuzz(
        &self,
        threat_model: &[u8],
        max_classes: usize,
    ) -> Result<UltraFuzzRoute<'_>, String> {
        if max_classes == 0 {
            return Err("max classes must be positive".into());
        }
        let model = serde_json::from_slice::<ThreatModel>(threat_model)
            .map_err(|error| format!("invalid threat model: {error}"))?;
        if model.schema_version != "ultrafuzz.threat-model.v1" {
            return Err(format!(
                "unsupported threat model schema: {}",
                model.schema_version
            ));
        }
        let queries = model.queries();
        if queries.is_empty() {
            return Err("threat model has no routing queries".into());
        }

        let mut scores = HashMap::<&str, f64>::new();
        let mut evidence = HashMap::<&str, Vec<UltraFuzzRouteEvidence>>::new();
        for query in &queries {
            for (index, hit) in self
                .rank(&query.text, &[], RetrievalMode::Bm25)
                .into_iter()
                .take(RESULTS_PER_QUERY)
                .enumerate()
            {
                *scores.entry(hit.id).or_default() +=
                    query.weight / (RRF_OFFSET + index + 1) as f64;
                evidence
                    .entry(hit.id)
                    .or_default()
                    .push(UltraFuzzRouteEvidence {
                        query_id: query.id.clone(),
                        rank: index + 1,
                        score: hit.score,
                    });
            }
        }
        let mut ranked = scores.into_iter().collect::<Vec<_>>();
        ranked.sort_unstable_by(|(left_id, left), (right_id, right)| {
            right.total_cmp(left).then(left_id.cmp(right_id))
        });
        let selected = ranked
            .into_iter()
            .take(max_classes)
            .map(|(id, score)| {
                let mut evidence = evidence.remove(id).expect("a score has routing evidence");
                evidence.sort_unstable_by(|left, right| {
                    left.query_id
                        .cmp(&right.query_id)
                        .then(left.rank.cmp(&right.rank))
                });
                UltraFuzzRouteSelection {
                    id,
                    score,
                    evidence,
                }
            })
            .collect();
        Ok(UltraFuzzRoute {
            schema: "buggraph/ultrafuzz-route-v1",
            corpus_revision: &self.corpus().revision,
            threat_model_sha256: format!("{:x}", Sha256::digest(threat_model)),
            query_count: queries.len(),
            max_classes,
            selected,
        })
    }
}

impl ThreatModel {
    fn queries(&self) -> Vec<Query> {
        let invariants = self
            .invariants
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect::<HashMap<_, _>>();
        let surfaces = self
            .attack_surfaces
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect::<HashMap<_, _>>();
        let mut queries = self
            .threats
            .iter()
            .filter_map(|threat| {
                let mut parts = vec![
                    threat.title.as_str(),
                    threat.description.as_str(),
                    threat.impact.as_str(),
                ];
                parts.extend(threat.preconditions.iter().map(String::as_str));
                for invariant in threat
                    .invariant_ids
                    .iter()
                    .filter_map(|id| invariants.get(id.as_str()))
                {
                    parts.extend([invariant.name.as_str(), invariant.statement.as_str()]);
                }
                for surface in threat
                    .attack_surface_ids
                    .iter()
                    .filter_map(|id| surfaces.get(id.as_str()))
                {
                    parts.extend([surface.name.as_str(), surface.description.as_str()]);
                    parts.extend(surface.entry_points.iter().map(String::as_str));
                }
                query(format!("threat:{}", threat.id), THREAT_WEIGHT, parts)
            })
            .collect::<Vec<_>>();
        queries.extend(self.invariants.iter().filter_map(|item| {
            query(
                format!("invariant:{}", item.id),
                INVARIANT_WEIGHT,
                [item.name.as_str(), item.statement.as_str()],
            )
        }));
        queries.extend(self.coverage_gaps.iter().filter_map(|item| {
            query(
                format!("gap:{}", item.id),
                COVERAGE_GAP_WEIGHT,
                [
                    item.name.as_str(),
                    item.title.as_str(),
                    item.description.as_str(),
                    item.reason.as_str(),
                ],
            )
        }));
        queries
    }
}

fn query<'a>(id: String, weight: f64, parts: impl IntoIterator<Item = &'a str>) -> Option<Query> {
    let text = parts
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!text.is_empty()).then_some(Query { id, weight, text })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corpus, Kind, Node};
    use serde_json::json;

    fn graph() -> Graph {
        Graph::compile(Corpus {
            revision: "test-v1".into(),
            sources: Vec::new(),
            nodes: [
                ("access", "privileged authorization role bypass"),
                ("rounding", "vault share rounding accounting precision loss"),
                ("reentrancy", "callback reentrancy external call"),
            ]
            .into_iter()
            .map(|(id, summary)| Node {
                id: id.into(),
                kind: Kind::FailureMode,
                summary: summary.into(),
                definition: String::new(),
                facets: Vec::new(),
                exclusions: Vec::new(),
                sources: Vec::new(),
                applicability: Vec::new(),
                mappings: Vec::new(),
                review: Default::default(),
                code: Vec::new(),
            })
            .collect(),
            edges: Vec::new(),
        })
        .unwrap()
    }

    #[test]
    fn routes_linked_threat_model_fields_and_rejects_invalid_inputs() {
        let bytes = serde_json::to_vec(&json!({
            "schema_version": "ultrafuzz.threat-model.v1",
            "attack_surfaces": [{
                "id": "surface:vault", "name": "Vault shares",
                "description": "Share accounting", "entry_points": ["deposit", "redeem"]
            }],
            "invariants": [{
                "id": "invariant:rounding", "name": "Rounding",
                "statement": "Vault share precision is conserved"
            }],
            "threats": [{
                "id": "threat:dilution", "title": "Share dilution",
                "description": "Rounding changes accounting", "impact": "Precision loss",
                "preconditions": ["Small deposit"],
                "invariant_ids": ["invariant:rounding"],
                "attack_surface_ids": ["surface:vault"]
            }],
            "coverage_gaps": []
        }))
        .unwrap();
        let graph = graph();
        let route = graph.route_ultrafuzz(&bytes, 2).unwrap();
        assert_eq!(route.schema, "buggraph/ultrafuzz-route-v1");
        assert_eq!(route.query_count, 2);
        assert_eq!(route.selected[0].id, "rounding");
        assert_eq!(route.selected.len(), 1);
        assert_eq!(route.threat_model_sha256.len(), 64);
        assert!(graph.route_ultrafuzz(&bytes, 0).is_err());

        let wrong = br#"{"schema_version":"ultrafuzz.threat-model.v2"}"#;
        assert!(graph.route_ultrafuzz(wrong, 1).is_err());
    }
}
