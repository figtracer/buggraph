//! Local JSON interface for taxonomy validation and knowledge retrieval.

use buggraph::{
    BundleFormat, BundleOptions, Corpus, Detail, EvalSuite, Graph, Ledger, RetrievalMode,
    TokenCounter, expand_bundle,
};
use std::{
    env,
    error::Error,
    fs,
    io::{self, Write},
    process::ExitCode,
};

const USAGE: &str = "Usage: buggraph validate CORPUS\n       buggraph context CORPUS MAX_BYTES [dimension:value ...]\n       buggraph search CORPUS MODE MODEL MAX_TOKENS QUERY [dimension:value ...]\n       buggraph bundle CORPUS MODE MODEL MAX_TOKENS DETAIL QUERY [dimension:value ...] [--compact]\n       buggraph expand BUNDLE_JSON\n       buggraph eval CORPUS SUITE MODEL MAX_TOKENS K\n       buggraph show CORPUS ID\n       buggraph descendants CORPUS ID\n       buggraph coverage CORPUS LEDGER\nModes: id_order, bm25, bm25_ancestors\nDetail: summary, full";

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
    let output = match args[0].as_str() {
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
        "show" if args.len() == 3 => {
            let node = graph.node(&args[2]).ok_or("unknown ID")?;
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
            serde_json::json!({"revision": graph.corpus().revision, "node": node, "edges": edges, "sources": sources})
        }
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
