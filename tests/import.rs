use buggraph::{Graph, import_owasp};
use std::{fs, path::PathBuf};

fn temporary_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("buggraph-import-{}-{name}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    root
}

#[test]
fn imports_exact_markdown_in_stable_id_order() {
    let root = temporary_root("exact");
    let first_dir = root.join("docs/SCWE/SCSVS-AUTH");
    let second_dir = root.join("docs/SCWE/SCSVS-ARCH");
    fs::create_dir_all(&first_dir).unwrap();
    fs::create_dir_all(&second_dir).unwrap();
    let first = "---\ntitle: First record\nid: SCWE-002\n---\n\n## Description\nExact β.\n\n```solidity\ncontract A {}\n```\n";
    let second = "---\r\ntitle: Second record\r\nid: SCWE-001\r\n---\r\n\r\nExact CRLF.\r\n";
    fs::write(first_dir.join("SCWE-002.md"), first).unwrap();
    fs::write(second_dir.join("SCWE-001.md"), second).unwrap();

    let corpus = import_owasp(&root, "a".repeat(40).as_str()).unwrap();
    assert_eq!(
        corpus
            .nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        ["scwe:001", "scwe:002"]
    );
    assert_eq!(corpus.nodes[0].definition, second);
    assert_eq!(corpus.nodes[1].definition, first);
    assert_eq!(corpus.nodes[0].summary, "Second record");
    assert_eq!(corpus.nodes[1].summary, "First record: Exact β.");
    assert_eq!(corpus.revision, "owasp-scwe-aaaaaaaa-source-v2");
    assert_eq!(
        corpus.sources[0].url,
        format!(
            "https://github.com/OWASP/owasp-scs/blob/{}/docs/SCWE/SCSVS-ARCH/SCWE-001.md",
            "a".repeat(40)
        )
    );
    assert!(Graph::compile(corpus).is_ok());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_unpinned_or_mismatched_sources() {
    let root = temporary_root("reject");
    let directory = root.join("docs/SCWE/SCSVS-AUTH");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("SCWE-001.md"),
        "---\ntitle: Wrong ID\nid: SCWE-002\n---\nBody\n",
    )
    .unwrap();
    assert!(import_owasp(&root, "main").is_err());
    assert!(import_owasp(&root, &"a".repeat(40)).is_err());
    fs::remove_dir_all(root).unwrap();
}
