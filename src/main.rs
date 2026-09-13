//! Local JSON interface for taxonomy validation and knowledge retrieval.

use buggraph::{
    BundleFormat, BundleOptions, Corpus, Detail, EvalSuite, Graph, Ledger, RetrievalMode,
    TokenCounter, expand_bundle,
};
use serde_json::Value;
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufRead, Write},
    process::ExitCode,
};

const USAGE: &str = "Usage: buggraph validate CORPUS\n       buggraph context CORPUS MAX_BYTES [dimension:value ...]\n       buggraph search CORPUS MODE MODEL MAX_TOKENS QUERY [dimension:value ...]\n       buggraph bundle CORPUS MODE MODEL MAX_TOKENS DETAIL QUERY [dimension:value ...] [--compact]\n       buggraph explore CORPUS MODEL MAX_TOKENS DETAIL MAX_DIRECT DEPTH QUERY [dimension:value ...] [--compact]\n       buggraph serve CORPUS MODEL\n       buggraph expand BUNDLE_JSON\n       buggraph eval CORPUS SUITE MODEL MAX_TOKENS K\n       buggraph show CORPUS ID\n       buggraph descendants CORPUS ID\n       buggraph coverage CORPUS LEDGER\nModes: id_order, bm25, bm25_ancestors\nDetail: summary, full";

#[derive(serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum ServeRequest {
    Bundle {
        version: u8,
        id: String,
        mode: RetrievalMode,
        max_tokens: usize,
        detail: Detail,
        query: String,
        #[serde(default)]
        facets: Vec<String>,
        #[serde(default)]
        compact: bool,
    },
    Show {
        version: u8,
        id: String,
        record_id: String,
    },
    Explore {
        version: u8,
        id: String,
        max_tokens: usize,
        detail: Detail,
        query: String,
        max_direct_records: usize,
        max_depth: usize,
        #[serde(default)]
        facets: Vec<String>,
        #[serde(default)]
        compact: bool,
    },
    Descendants {
        version: u8,
        id: String,
        root_id: String,
        max_depth: usize,
    },
}

impl ServeRequest {
    fn version(&self) -> u8 {
        match self {
            Self::Bundle { version, .. }
            | Self::Show { version, .. }
            | Self::Explore { version, .. }
            | Self::Descendants { version, .. } => *version,
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::Bundle { id, .. }
            | Self::Show { id, .. }
            | Self::Explore { id, .. }
            | Self::Descendants { id, .. } => id,
        }
    }
}

fn context_response(id: String, context: buggraph::RankedContext<'_>) -> Value {
    serde_json::json!({
        "version": 1,
        "id": id,
        "ok": true,
        "context": context.jsonl,
        "tokens": context.tokens,
        "selected": context.hits,
        "omitted": context.omitted,
    })
}

fn exploration_response(id: String, result: buggraph::ExplorationContext<'_>) -> Value {
    let selected = result
        .context
        .hits
        .iter()
        .zip(&result.depths)
        .map(|(hit, depth)| {
            serde_json::json!({
                "id": hit.id,
                "score": hit.score,
                "relation": hit.relation,
                "depth": depth,
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "version": 1,
        "id": id,
        "ok": true,
        "context": result.context.jsonl,
        "tokens": result.context.tokens,
        "selected": selected,
        "omitted": result.context.omitted,
    })
}

fn show_value(graph: &Graph, id: &str) -> Option<Value> {
    let node = graph.node(id)?;
    let edges = graph
        .corpus()
        .edges
        .iter()
        .filter(|edge| edge.from == node.id || edge.to == node.id)
        .collect::<Vec<_>>();
    let sources = graph
        .corpus()
        .sources
        .iter()
        .filter(|source| node.sources.contains(&source.id))
        .collect::<Vec<_>>();
    Some(serde_json::json!({
        "revision": graph.corpus().revision,
        "node": node,
        "edges": edges,
        "sources": sources,
    }))
}

fn serve(graph: &Graph, counter: &TokenCounter) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    writeln!(
        stdout,
        "{}",
        serde_json::json!({
            "version": 1,
            "ready": true,
            "revision": graph.corpus().revision,
            "model": counter.model(),
        })
    )?;
    stdout.flush()?;

    for line in stdin.lock().lines() {
        let value = line.map_err(|error| error.to_string()).and_then(|line| {
            serde_json::from_str::<Value>(&line).map_err(|error| error.to_string())
        });
        let request_id = value
            .as_ref()
            .ok()
            .and_then(|value| value.get("id"))
            .cloned();
        let request = value.and_then(|value| {
            serde_json::from_value::<ServeRequest>(value).map_err(|error| error.to_string())
        });
        let response = match request {
            Ok(request) if request.version() != 1 => serde_json::json!({
                "version": 1, "id": request.id(), "ok": false, "error": "unsupported request version"
            }),
            Ok(request) => match request {
                ServeRequest::Bundle {
                    id,
                    mode,
                    max_tokens,
                    detail,
                    query,
                    facets,
                    compact,
                    ..
                } => {
                    let facets = facets.iter().map(String::as_str).collect::<Vec<_>>();
                    let context = graph.bundle_with_options(
                        &query,
                        &facets,
                        counter,
                        BundleOptions {
                            mode,
                            detail,
                            max_tokens,
                            format: if compact {
                                BundleFormat::Compact
                            } else {
                                BundleFormat::Json
                            },
                        },
                    );
                    context_response(id, context)
                }
                ServeRequest::Show { id, record_id, .. } => match show_value(graph, &record_id) {
                    Some(record) => serde_json::json!({
                        "version": 1, "id": id, "ok": true, "record": record
                    }),
                    None => serde_json::json!({
                        "version": 1, "id": id, "ok": false, "error": "unknown ID"
                    }),
                },
                ServeRequest::Explore {
                    id,
                    max_tokens,
                    detail,
                    query,
                    max_direct_records,
                    max_depth,
                    facets,
                    compact,
                    ..
                } => {
                    let facets = facets.iter().map(String::as_str).collect::<Vec<_>>();
                    let context = graph.bundle_direct_first(
                        &query,
                        &facets,
                        counter,
                        BundleOptions {
                            mode: RetrievalMode::Bm25,
                            detail,
                            max_tokens,
                            format: if compact {
                                BundleFormat::Compact
                            } else {
                                BundleFormat::Json
                            },
                        },
                        max_direct_records,
                        max_depth,
                    );
                    exploration_response(id, context)
                }
                ServeRequest::Descendants {
                    id,
                    root_id,
                    max_depth,
                    ..
                } => match graph.descendants_to_depth(&root_id, max_depth) {
                    Ok(records) => serde_json::json!({
                        "version": 1, "id": id, "ok": true, "records": records
                    }),
                    Err(error) => serde_json::json!({
                        "version": 1, "id": id, "ok": false, "error": error
                    }),
                },
            },
            Err(error) => serde_json::json!({
                "version": 1,
                "id": request_id.unwrap_or(Value::Null),
                "ok": false,
                "error": error,
            }),
        };
        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "--help") {
        writeln!(io::stdout().lock(), "{USAGE}")?;
        return Ok(());
    }
    if args.len() < 2 {
        return Err(USAGE.into());
    }
    if args[0] == "expand" && args.len() == 2 {
        let value = expand_bundle(&fs::read_to_string(&args[1])?)?;
        writeln!(io::stdout().lock(), "{value}")?;
        return Ok(());
    }
    let corpus = serde_json::from_slice::<Corpus>(&fs::read(&args[1])?)?;
    let graph = Graph::compile(corpus)?;
    if args[0] == "serve" && args.len() == 3 {
        let counter = TokenCounter::for_model(&args[2])?;
        return serve(&graph, &counter);
    }
    let output = match args[0].as_str() {
        "explore" if args.len() >= 8 => {
            let counter = TokenCounter::for_model(&args[2])?;
            let max_tokens = args[3].parse::<usize>()?;
            let detail =
                serde_json::from_value::<Detail>(serde_json::Value::String(args[4].clone()))?;
            let max_direct_records = args[5].parse::<usize>()?;
            let max_depth = args[6].parse::<usize>()?;
            let compact = args.len() > 8 && args.last().is_some_and(|arg| arg == "--compact");
            let facet_end = args.len() - usize::from(compact);
            let facets = args[8..facet_end]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let result = graph.bundle_direct_first(
                &args[7],
                &facets,
                &counter,
                BundleOptions {
                    mode: RetrievalMode::Bm25,
                    detail,
                    max_tokens,
                    format: if compact {
                        BundleFormat::Compact
                    } else {
                        BundleFormat::Json
                    },
                },
                max_direct_records,
                max_depth,
            );
            io::stdout()
                .lock()
                .write_all(result.context.jsonl.as_bytes())?;
            return Ok(());
        }
        "search" | "bundle" if args.len() >= 6 => {
            let mode = serde_json::from_value::<RetrievalMode>(serde_json::Value::String(
                args[2].clone(),
            ))?;
            let counter = TokenCounter::for_model(&args[3])?;
            let max_tokens = args[4].parse::<usize>()?;
            let bundled = args[0] == "bundle";
            let (query, facet_start, detail) = if bundled {
                if args.len() < 7 {
                    return Err(USAGE.into());
                }
                let detail =
                    serde_json::from_value::<Detail>(serde_json::Value::String(args[5].clone()))?;
                (&args[6], 7, detail)
            } else {
                (&args[5], 6, Detail::Summary)
            };
            let compact = bundled
                && args.len() > facet_start
                && args.last().is_some_and(|arg| arg == "--compact");
            let facet_end = args.len() - usize::from(compact);
            let facets = args[facet_start..facet_end]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let context = if bundled {
                graph.bundle_with_options(
                    query,
                    &facets,
                    &counter,
                    BundleOptions {
                        mode,
                        detail,
                        max_tokens,
                        format: if compact {
                            BundleFormat::Compact
                        } else {
                            BundleFormat::Json
                        },
                    },
                )
            } else {
                graph.ranked_context(query, &facets, mode, &counter, max_tokens, usize::MAX)
            };
            io::stdout().lock().write_all(context.jsonl.as_bytes())?;
            if bundled {
                return Ok(());
            }
            writeln!(
                io::stderr().lock(),
                "{}",
                serde_json::json!({"model": counter.model(), "tokens": context.tokens, "selected": context.hits, "omitted": context.omitted})
            )?;
            return Ok(());
        }
        "eval" if args.len() == 6 => {
            let suite = serde_json::from_slice::<EvalSuite>(&fs::read(&args[2])?)?;
            let counter = TokenCounter::for_model(&args[3])?;
            serde_json::to_value(graph.evaluate(
                &suite,
                &counter,
                args[4].parse()?,
                args[5].parse()?,
            )?)?
        }
        "validate" if args.len() == 2 => {
            serde_json::json!({"revision": graph.corpus().revision, "nodes": graph.corpus().nodes.len(), "edges": graph.corpus().edges.len()})
        }
        "show" if args.len() == 3 => show_value(&graph, &args[2]).ok_or("unknown ID")?,
        "descendants" if args.len() == 3 => serde_json::to_value(graph.descendants(&args[2])?)?,
        "context" if args.len() >= 3 => {
            let max_bytes = args[2].parse::<usize>()?;
            let facets = args[3..].iter().map(String::as_str).collect::<Vec<_>>();
            let context = graph.context(&facets, max_bytes);
            io::stdout().lock().write_all(context.jsonl.as_bytes())?;
            writeln!(
                io::stderr().lock(),
                "selected={} omitted={} bytes={}",
                context.selected,
                context.omitted,
                context.jsonl.len()
            )?;
            return Ok(());
        }
        "coverage" if args.len() == 3 => {
            let ledger = serde_json::from_slice::<Ledger>(&fs::read(&args[2])?)?;
            serde_json::to_value(graph.coverage(&ledger)?)?
        }
        _ => return Err(USAGE.into()),
    };
    writeln!(
        io::stdout().lock(),
        "{}",
        serde_json::to_string_pretty(&output)?
    )?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "{error}");
            ExitCode::FAILURE
        }
    }
}
