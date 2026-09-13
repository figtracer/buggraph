use buggraph::{
    BundleFormat, BundleOptions, Corpus, Detail, EvalSuite, Graph, RetrievalMode, TokenCounter,
};
use serde_json::{Value, json};
use std::process::Command;

fn graph() -> Graph {
    Graph::compile(serde_json::from_str::<Corpus>(include_str!("../data/curated.json")).unwrap())
        .unwrap()
}

#[test]
fn lexical_ranking_respects_facets_and_returns_no_match_for_unknown_terms() {
    let graph = graph();
    let hits = graph.rank("stale oracle response freshness", &[], RetrievalMode::Bm25);
    assert_eq!(hits[0].id, "fm:oracle-response-validity");
    assert!(graph.rank("zxqvjk", &[], RetrievalMode::Bm25).is_empty());
    let hits = graph.rank(
        "oracle response freshness",
        &["component:oracle"],
        RetrievalMode::Bm25Ancestors,
    );
    assert!(!hits.is_empty());
    assert!(hits.iter().all(|hit| {
        graph
            .node(hit.id)
            .unwrap()
            .facets
            .contains(&"component:oracle".into())
    }));
    let hits = graph.rank("gas collection", &[], RetrievalMode::Bm25Ancestors);
    let mut ids = hits.iter().map(|hit| hit.id).collect::<Vec<_>>();
    let count = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(count, ids.len());
    assert!(hits.iter().any(|hit| hit.relation == "ancestor"));
}

#[test]
fn direct_first_expansion_is_bounded_deduplicated_and_preserves_matches() {
    let corpus = json!({"revision":"depth-v1", "nodes":[
        {"id":"root","kind":"failure_mode","summary":"broad category with a very long structural explanation that should not displace direct evidence"},
        {"id":"left","kind":"failure_mode","summary":"left category"},
        {"id":"other-parent","kind":"failure_mode","summary":"separate category"},
        {"id":"right","kind":"failure_mode","summary":"right category"},
        {"id":"leaf","kind":"failure_mode","summary":"needle concrete mechanism"},
        {"id":"other","kind":"failure_mode","summary":"needle separate mechanism"}
    ], "edges":[
        {"from":"leaf","relation":"specializes","to":"left"},
        {"from":"leaf","relation":"specializes","to":"right"},
        {"from":"left","relation":"specializes","to":"root"},
        {"from":"right","relation":"specializes","to":"root"},
        {"from":"other","relation":"specializes","to":"other-parent"}
    ]});
    let graph = Graph::compile(serde_json::from_value(corpus).unwrap()).unwrap();
    let counter = TokenCounter::for_model("gpt-4o").unwrap();
    let options = |max_tokens| BundleOptions {
        mode: RetrievalMode::Bm25,
        detail: Detail::Summary,
        format: BundleFormat::Json,
        max_tokens,
    };

    let flat = graph.bundle_direct_first("needle", &[], &counter, options(usize::MAX), 2, 0);
    assert_eq!(
        flat.context
            .hits
            .iter()
            .map(|hit| hit.id)
            .collect::<Vec<_>>(),
        ["leaf", "other"]
    );
    assert_eq!(flat.depths, [0, 0]);

    let depth_one = graph.bundle_direct_first("needle", &[], &counter, options(usize::MAX), 2, 1);
    assert_eq!(
        depth_one
            .context
            .hits
            .iter()
            .zip(&depth_one.depths)
            .filter(|(hit, _)| hit.relation == "ancestor")
            .map(|(hit, depth)| (hit.id, *depth))
            .collect::<Vec<_>>(),
        [("left", 1), ("right", 1), ("other-parent", 1)]
    );

    let depth_two = graph.bundle_direct_first("needle", &[], &counter, options(usize::MAX), 1, 2);
    let ancestors = depth_two
        .context
        .hits
        .iter()
        .zip(&depth_two.depths)
        .filter(|(hit, _)| hit.relation == "ancestor")
        .map(|(hit, depth)| (hit.id, *depth))
        .collect::<Vec<_>>();
    assert_eq!(ancestors, [("left", 1), ("right", 1), ("root", 2)]);
    assert!(
        !depth_two
            .context
            .hits
            .iter()
            .any(|hit| hit.id == "other-parent")
    );

    let tight =
        graph.bundle_direct_first("needle", &[], &counter, options(flat.context.tokens), 2, 2);
    assert_eq!(
        tight
            .context
            .hits
            .iter()
            .filter(|hit| hit.relation == "match")
            .map(|hit| hit.id)
            .collect::<Vec<_>>(),
        ["leaf", "other"]
    );
}

#[test]
fn token_budgets_recount_the_serialized_context_for_both_encodings() {
    let graph = graph();
    for model in ["gpt-4", "gpt-4o"] {
        let counter = TokenCounter::for_model(model).unwrap();
        assert_eq!(counter.count("hello world"), 2);
        assert!(counter.count("é 🦀 <|endoftext|>") > 0);
        for budget in [0, 1, 64, 128, 256, 1024] {
            let result = graph.ranked_context(
                "oracle response freshness",
                &[],
                RetrievalMode::Bm25Ancestors,
                &counter,
                budget,
                3,
            );
            assert!(result.tokens <= budget);
            assert_eq!(counter.count(&result.jsonl), result.tokens);
            assert_eq!(result.hits.len(), result.jsonl.lines().count());
            assert!(result.hits.len() <= 3);
            for line in result.jsonl.lines() {
                let value = serde_json::from_str::<Value>(line).unwrap();
                assert_eq!(value["revision"], "scwe-starter-v1");
            }
        }
        assert!(
            graph
                .ranked_context("oracle", &[], RetrievalMode::Bm25, &counter, usize::MAX, 0)
                .jsonl
                .is_empty()
        );
    }
    assert!(TokenCounter::for_model("unknown-model-zxqvjk").is_err());
}

#[test]
fn evaluation_rejects_leakage_stale_revisions_and_bad_labels() {
    let graph = graph();
    let counter = TokenCounter::for_model("gpt-4o").unwrap();
    let base = json!({"revision":"test-v1", "corpus_revision":"scwe-starter-v1", "description":"test", "cases":[
        {"id":"a","group":"oracle","split":"dev","query":"oracle freshness","relevant_ids":["fm:oracle-response-validity"]},
        {"id":"b","group":"oracle","split":"test","query":"price validity","relevant_ids":["fm:oracle-price-integrity"]}
    ]});
    let suite = serde_json::from_value::<EvalSuite>(base.clone()).unwrap();
    assert!(
        graph
            .evaluate(&suite, &counter, 1024, 3)
            .unwrap_err()
            .contains("across splits")
    );
    let mut stale = base.clone();
    stale["corpus_revision"] = json!("wrong");
    assert!(
        graph
            .evaluate(&serde_json::from_value(stale).unwrap(), &counter, 1024, 3)
            .is_err()
    );
    let mut invalid = base.clone();
    invalid["cases"][0]["relevant_ids"] = json!(["missing"]);
    assert!(
        graph
            .evaluate(&serde_json::from_value(invalid).unwrap(), &counter, 1024, 3)
            .is_err()
    );
    let mut duplicate = base;
    duplicate["cases"][1]["query"] = json!("ORACLE freshness!");
    assert!(
        graph
            .evaluate(
                &serde_json::from_value(duplicate).unwrap(),
                &counter,
                1024,
                3
            )
            .is_err()
    );
}

#[test]
fn metrics_match_a_hand_calculated_ranking() {
    let corpus = json!({"revision":"tiny", "nodes":[
        {"id":"a","kind":"failure_mode","summary":"alpha"},
        {"id":"b","kind":"failure_mode","summary":"beta"},
        {"id":"c","kind":"failure_mode","summary":"gamma"}
    ], "edges":[]});
    let graph = Graph::compile(serde_json::from_value(corpus).unwrap()).unwrap();
    let suite = serde_json::from_value::<EvalSuite>(json!({
        "revision":"tiny-eval","corpus_revision":"tiny","description":"hand calculation", "cases":[
            {"id":"q","group":"g","split":"test","query":"beta gamma","relevant_ids":["b","c"]}
        ]
    }))
    .unwrap();
    let counter = TokenCounter::for_model("gpt-4o").unwrap();
    let report = serde_json::to_value(graph.evaluate(&suite, &counter, 1024, 2).unwrap()).unwrap();
    let row = &report["results"][0]["cases"][0];
    assert_eq!(row["retrieved"], json!(["a", "b"]));
    assert_eq!(row["recall"], 0.5);
    assert_eq!(row["precision"], 0.5);
    assert_eq!(row["reciprocal_rank"], 0.5);
    let expected = (1.0 / 3_f64.log2()) / (1.0 + 1.0 / 3_f64.log2());
    assert!((row["ndcg"].as_f64().unwrap() - expected).abs() < f64::EPSILON);
}

#[test]
fn source_checked_records_require_resolvable_provenance() {
    let mut corpus = serde_json::from_str::<Value>(include_str!("../data/curated.json")).unwrap();
    corpus["nodes"][0]["sources"] = json!(["unknown"]);
    assert!(Graph::compile(serde_json::from_value(corpus).unwrap()).is_err());
    let mut corpus = serde_json::from_str::<Value>(include_str!("../data/curated.json")).unwrap();
    corpus["nodes"][0]["sources"] = json!([]);
    assert!(Graph::compile(serde_json::from_value(corpus).unwrap()).is_err());
}

#[test]
fn search_cli_obeys_budget_and_unknown_model_fails() {
    let bin = env!("CARGO_BIN_EXE_buggraph");
    let output = Command::new(bin)
        .args([
            "search",
            "data/curated.json",
            "bm25",
            "gpt-4o",
            "256",
            "stale oracle response",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let meta = serde_json::from_slice::<Value>(&output.stderr).unwrap();
    let counter = TokenCounter::for_model("gpt-4o").unwrap();
    let count = counter.count(std::str::from_utf8(&output.stdout).unwrap());
    assert!(count <= 256);
    assert_eq!(meta["tokens"], count);
    let output = Command::new(bin)
        .args([
            "search",
            "data/curated.json",
            "bm25",
            "unknown-model-zxqvjk",
            "256",
            "oracle",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn bundles_preserve_details_citations_and_edges_with_exact_budgets() {
    let corpus = json!({"revision":"bundle-v1", "sources":[
        {"id":"s","title":"Reference","url":"https://example.org/reference","revision":"1","license":"MIT"}
    ], "nodes":[
        {"id":"a","kind":"failure_mode","summary":"alpha parent", "sources":["s"],"review":"imported"},
        {"id":"b","kind":"failure_mode","summary":"alpha child", "sources":["s"],
         "definition":"A complete description: é 🦀 <|endoftext|>.","applicability":["condition"],
         "exclusions":["boundary"],"mappings":["external:1"]}
    ], "edges":[{"from":"b","relation":"specializes","to":"a"}]});
    let graph = Graph::compile(serde_json::from_value(corpus.clone()).unwrap()).unwrap();
    for model in ["gpt-4", "gpt-4o"] {
        let counter = TokenCounter::for_model(model).unwrap();
        let all = graph.bundle(
            "alpha",
            &[],
            RetrievalMode::Bm25,
            Detail::Full,
            &counter,
            usize::MAX,
        );
        let value = serde_json::from_str::<Value>(&all.jsonl).unwrap();
        assert_eq!(value["sources"], json!(["https://example.org/reference"]));
        assert_eq!(value["edges"], corpus["edges"]);
        for record in value["records"].as_array().unwrap() {
            let node = graph.node(record["id"].as_str().unwrap()).unwrap();
            assert_eq!(record["sources"], json!([0]));
            if !node.definition.is_empty() {
                assert_eq!(record["definition"], node.definition);
                assert_eq!(record["applicability"], json!(node.applicability));
                assert_eq!(record["exclusions"], json!(node.exclusions));
                assert_eq!(record["mappings"], json!(node.mappings));
            }
        }
        for detail in [Detail::Summary, Detail::Full] {
            for budget in [0, 1, 64, 128, all.tokens - 1, all.tokens] {
                let result =
                    graph.bundle("alpha", &[], RetrievalMode::Bm25, detail, &counter, budget);
                assert!(result.tokens <= budget);
                assert_eq!(result.tokens, counter.count(&result.jsonl));
                assert_eq!(result.hits.len() + result.omitted, 2);
                if !result.jsonl.is_empty() {
                    let value = serde_json::from_str::<Value>(&result.jsonl).unwrap();
                    assert_eq!(
                        value["records"].as_array().unwrap().len(),
                        result.hits.len()
                    );
                    if result.hits.len() == 1 {
                        assert!(value.get("edges").is_none());
                    }
                    if matches!(detail, Detail::Summary) {
                        assert!(
                            value["records"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .all(|record| record.get("definition").is_none())
                        );
                    }
                }
            }
        }
        let exact = graph.bundle(
            "alpha",
            &[],
            RetrievalMode::Bm25,
            Detail::Full,
            &counter,
            all.tokens,
        );
        assert_eq!(exact.jsonl, all.jsonl);
        assert!(
            graph
                .bundle(
                    "zxqv",
                    &[],
                    RetrievalMode::Bm25,
                    Detail::Full,
                    &counter,
                    1024
                )
                .jsonl
                .is_empty()
        );
    }
    let mut missing = corpus;
    missing["nodes"][0]["sources"] = json!([]);
    assert!(Graph::compile(serde_json::from_value(missing).unwrap()).is_err());
}

#[test]
fn reference_corpus_and_bundle_cli_return_pinned_descriptions() {
    let graph =
        Graph::compile(serde_json::from_str::<Corpus>(include_str!("../data/owasp.json")).unwrap())
            .unwrap();
    assert_eq!(graph.corpus().nodes.len(), 156);
    assert_eq!(graph.corpus().sources.len(), 156);
    assert!(graph.corpus().edges.is_empty());
    assert!(
        graph
            .corpus()
            .nodes
            .iter()
            .all(|node| !node.definition.is_empty()
                && matches!(node.review, buggraph::ReviewStatus::Imported))
    );
    let output = Command::new(env!("CARGO_BIN_EXE_buggraph"))
        .args([
            "bundle",
            "data/owasp.json",
            "bm25",
            "gpt-4o",
            "1024",
            "full",
            "contract architecture",
            "category:scsvs-arch",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = serde_json::from_slice::<Value>(&output.stdout).unwrap();
    assert!(output.stderr.is_empty());
    let counter = TokenCounter::for_model("gpt-4o").unwrap();
    let tokens = counter.count(std::str::from_utf8(&output.stdout).unwrap());
    assert!(tokens <= 1024);
    assert!(value["omitted"].is_u64());
    for record in value["records"].as_array().unwrap() {
        let node = graph.node(record["id"].as_str().unwrap()).unwrap();
        assert_eq!(record["definition"], node.definition);
        assert_eq!(record["facets"], json!(node.facets));
        let source = &graph
            .corpus()
            .sources
            .iter()
            .find(|source| source.id == node.sources[0])
            .unwrap();
        let slot = record["sources"][0].as_u64().unwrap() as usize;
        assert_eq!(value["sources"][slot], source.url);
    }
}
