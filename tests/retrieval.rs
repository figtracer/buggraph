use buggraph::{Corpus, EvalSuite, Graph, RetrievalMode, TokenCounter};
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
