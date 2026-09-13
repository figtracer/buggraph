//! Local JSON interface for taxonomy validation and knowledge retrieval.

use buggraph::{Corpus, Graph, Ledger};
use std::{
    env,
    error::Error,
    fs,
    io::{self, Write},
    process::ExitCode,
};

const USAGE: &str = "Usage: buggraph validate CORPUS\n       buggraph context CORPUS MAX_BYTES [dimension:value ...]\n       buggraph show CORPUS ID\n       buggraph descendants CORPUS ID\n       buggraph coverage CORPUS LEDGER";

fn run() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "--help") {
        writeln!(io::stdout().lock(), "{USAGE}")?;
        return Ok(());
    }
    if args.len() < 2 {
        return Err(USAGE.into());
    }
    let corpus = serde_json::from_slice::<Corpus>(&fs::read(&args[1])?)?;
    let graph = Graph::compile(corpus)?;
    let output = match args[0].as_str() {
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
            serde_json::json!({"revision": graph.corpus().revision, "node": node, "edges": edges})
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
