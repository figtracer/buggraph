//! Deterministic import of a pinned OWASP SCWE source tree.

use crate::{Corpus, Edge, Kind, Node, ReviewStatus, Source};
use std::{collections::HashSet, fs, path::Path};

const LICENSE: &str = "CC-BY-SA-4.0";

/// Import every SCWE Markdown source without inferring taxonomy edges.
pub fn import_owasp(root: &Path, revision: &str) -> Result<Corpus, String> {
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("OWASP revision must be a 40-character commit hash".into());
    }
    let scwe_root = root.join("docs/SCWE");
    let mut paths = Vec::new();
    for group in sorted_entries(&scwe_root)? {
        let file_type = group.file_type().map_err(|error| error.to_string())?;
        let group_name = group.file_name().to_string_lossy().into_owned();
        if !file_type.is_dir() || !group_name.starts_with("SCSVS-") {
            continue;
        }
        for entry in sorted_entries(&group.path())? {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry
                .file_type()
                .map_err(|error| error.to_string())?
                .is_file()
                && name.starts_with("SCWE-")
                && name.ends_with(".md")
            {
                paths.push((group_name.clone(), entry.path()));
            }
        }
    }
    paths.sort_unstable_by(|left, right| left.1.cmp(&right.1));
    if paths.is_empty() {
        return Err("OWASP source tree contains no SCWE Markdown files".into());
    }

    let mut sources = Vec::with_capacity(paths.len());
    let mut nodes = Vec::with_capacity(paths.len());
    let mut seen = HashSet::with_capacity(paths.len());
    for (group, path) in paths {
        let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let metadata = frontmatter(&text)?;
        let source_id = scalar(metadata, "id")?.to_owned();
        let title = scalar(metadata, "title")?.to_owned();
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or("invalid SCWE path")?;
        if source_id != stem {
            return Err(format!("SCWE ID/path mismatch: {}", path.display()));
        }
        let number = source_id
            .strip_prefix("SCWE-")
            .ok_or_else(|| format!("invalid SCWE ID: {source_id}"))?;
        if number.len() != 3 || !number.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(format!("invalid SCWE ID: {source_id}"));
        }
        if !seen.insert(source_id.clone()) {
            return Err(format!("duplicate SCWE ID: {source_id}"));
        }
        sources.push(Source {
            id: source_id.clone(),
            title: format!("{source_id}: {title}"),
            url: format!("https://scs.owasp.org/SCWE/view/{source_id}/"),
            revision: revision.to_owned(),
            license: LICENSE.into(),
        });
        nodes.push(Node {
            id: format!("scwe:{}", number.to_lowercase()),
            kind: Kind::FailureMode,
            summary: title,
            definition: text,
            facets: vec![format!("category:{}", group.to_lowercase())],
            exclusions: Vec::new(),
            sources: vec![source_id.clone()],
            applicability: Vec::new(),
            mappings: vec![source_id],
            review: ReviewStatus::Imported,
            code: Vec::new(),
        });
    }
    sources.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    nodes.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    Ok(Corpus {
        revision: format!("owasp-scwe-{}-source-v1", &revision[..8]),
        sources,
        nodes,
        edges: Vec::<Edge>::new(),
    })
}

fn sorted_entries(path: &Path) -> Result<Vec<fs::DirEntry>, String> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    entries.sort_unstable_by_key(fs::DirEntry::file_name);
    Ok(entries)
}

fn frontmatter(text: &str) -> Result<&str, String> {
    let (rest, delimiter) = if let Some(rest) = text.strip_prefix("---\n") {
        (rest, "\n---\n")
    } else if let Some(rest) = text.strip_prefix("---\r\n") {
        (rest, "\r\n---\r\n")
    } else {
        return Err("SCWE source is missing YAML frontmatter".into());
    };
    let end = rest
        .find(delimiter)
        .ok_or("SCWE source has unterminated YAML frontmatter")?;
    Ok(&rest[..end])
}

fn scalar<'a>(metadata: &'a str, key: &str) -> Result<&'a str, String> {
    let prefix = format!("{key}:");
    metadata
        .lines()
        .find_map(|line| line.strip_prefix(&prefix).map(str::trim))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("SCWE frontmatter requires {key}"))
}

#[cfg(test)]
mod tests {
    use super::{frontmatter, scalar};

    #[test]
    fn parses_lf_and_crlf_frontmatter_without_changing_source_text() {
        for text in [
            "---\nid: SCWE-001\ntitle: Example\n---\n\n## Description\nBody\n",
            "---\r\nid: SCWE-001\r\ntitle: Example\r\n---\r\n\r\nBody\r\n",
        ] {
            let metadata = frontmatter(text).unwrap();
            assert_eq!(scalar(metadata, "id").unwrap(), "SCWE-001");
            assert_eq!(scalar(metadata, "title").unwrap(), "Example");
            assert!(text.ends_with(if text.contains("\r\n") { "\r\n" } else { "\n" }));
        }
    }
}
