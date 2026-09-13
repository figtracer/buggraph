//! Deterministic import of a pinned OWASP SCWE source tree.

use crate::{Corpus, Edge, Kind, Node, ReviewStatus, Source};
use std::{collections::HashSet, fs, path::Path};

const LICENSE: &str = "CC-BY-SA-4.0";

struct ScsvsGroup {
    id: String,
    title: String,
    description: String,
}

/// Import every SCWE Markdown source and its explicit SCSVS hierarchy.
pub fn import_owasp(root: &Path, revision: &str) -> Result<Corpus, String> {
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("OWASP revision must be a 40-character commit hash".into());
    }
    let scsvs_path = root.join("docs/SCSVS/scsvs.yaml");
    let scsvs_text = fs::read_to_string(&scsvs_path).map_err(|error| error.to_string())?;
    let groups = scsvs_groups(&scsvs_text)?;
    let group_ids = groups
        .iter()
        .map(|group| group.id.to_lowercase())
        .collect::<HashSet<_>>();
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

    let mut sources = Vec::with_capacity(paths.len() + 1);
    sources.push(Source {
        id: "SCSVS".into(),
        title: "Smart Contract Security Verification Standard (SCSVS)".into(),
        url: format!("https://github.com/OWASP/owasp-scs/blob/{revision}/docs/SCSVS/scsvs.yaml"),
        revision: revision.to_owned(),
        license: LICENSE.into(),
    });
    let mut nodes = Vec::with_capacity(paths.len() + groups.len() + 1);
    nodes.push(Node {
        id: "taxonomy:scsvs".into(),
        kind: Kind::FailureMode,
        summary: "Smart contract security failure modes".into(),
        definition: String::new(),
        facets: vec!["taxonomy:root".into()],
        exclusions: Vec::new(),
        sources: vec!["SCSVS".into()],
        applicability: Vec::new(),
        mappings: vec!["SCSVS".into()],
        review: ReviewStatus::Imported,
        code: Vec::new(),
    });
    let mut edges = Vec::with_capacity(paths.len() + groups.len());
    for group in groups {
        let id = category_node_id(&group.id.to_lowercase())?;
        let title = category_title(&group.id, &group.title)?;
        let summary = if group.description.is_empty() || group.description == "TBD" {
            title.to_owned()
        } else {
            format!("{title}: {}", group.description)
        };
        nodes.push(Node {
            id: id.clone(),
            kind: Kind::FailureMode,
            summary,
            definition: String::new(),
            facets: vec![
                "taxonomy:category".into(),
                format!("category:{}", group.id.to_lowercase()),
            ],
            exclusions: Vec::new(),
            sources: vec!["SCSVS".into()],
            applicability: Vec::new(),
            mappings: vec![group.id],
            review: ReviewStatus::Imported,
            code: Vec::new(),
        });
        edges.push(Edge {
            from: id,
            relation: crate::Relation::Specializes,
            to: "taxonomy:scsvs".into(),
        });
    }
    let mut seen = HashSet::with_capacity(paths.len());
    for (group, path) in paths {
        let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let metadata = frontmatter(&text)?;
        let source_id = scalar(metadata, "id")?.to_owned();
        let title = scalar(metadata, "title")?.to_owned();
        let category = inline_list_item(metadata, "scsvs-cg")?.to_lowercase();
        if !group_ids.contains(&category) {
            return Err(format!(
                "SCWE frontmatter references unknown category {category}"
            ));
        }
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
            url: format!(
                "https://github.com/OWASP/owasp-scs/blob/{revision}/docs/SCWE/{group}/{source_id}.md"
            ),
            revision: revision.to_owned(),
            license: LICENSE.into(),
        });
        let id = format!("scwe:{}", number.to_lowercase());
        nodes.push(Node {
            id: id.clone(),
            kind: Kind::FailureMode,
            summary: semantic_summary(&title, &text),
            definition: text,
            facets: vec![format!("category:{category}")],
            exclusions: Vec::new(),
            sources: vec![source_id.clone()],
            applicability: Vec::new(),
            mappings: vec![source_id],
            review: ReviewStatus::Imported,
            code: Vec::new(),
        });
        edges.push(Edge {
            from: id,
            relation: crate::Relation::Specializes,
            to: category_node_id(&category)?,
        });
    }
    sources.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    nodes.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    Ok(Corpus {
        revision: format!("owasp-scwe-{}-source-v4", &revision[..8]),
        sources,
        nodes,
        edges,
    })
}

fn category_title<'a>(id: &str, source_title: &'a str) -> Result<&'a str, String> {
    if source_title != "TBD" {
        return Ok(source_title);
    }
    match id {
        "SCSVS-AUTH" => Ok("Authorization and access control"),
        "SCSVS-BLOCK" => Ok("Block and transaction properties"),
        "SCSVS-BRIDGE" => Ok("Cross-chain bridges"),
        "SCSVS-COMM" => Ok("Communication and external calls"),
        "SCSVS-COMP" => Ok("Arithmetic and computation"),
        "SCSVS-CRYPTO" => Ok("Cryptography"),
        "SCSVS-DEFI" => Ok("Decentralized finance"),
        "SCSVS-GOV" => Ok("Governance"),
        "SCSVS-ORACLE" => Ok("Oracles and price feeds"),
        _ => Err(format!("SCSVS group {id} has no usable title")),
    }
}

fn category_node_id(category: &str) -> Result<String, String> {
    category
        .strip_prefix("scsvs-")
        .map(|suffix| format!("scsvs:{suffix}"))
        .ok_or_else(|| format!("invalid SCSVS category {category}"))
}

fn scsvs_groups(text: &str) -> Result<Vec<ScsvsGroup>, String> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut groups = Vec::new();
    let mut seen = HashSet::new();
    let mut index = 0;
    while index < lines.len() {
        let Some(id) = lines[index].strip_prefix("- gid: ").map(str::trim) else {
            index += 1;
            continue;
        };
        if !id.starts_with("SCSVS-")
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte == b'-')
            || !seen.insert(id.to_owned())
        {
            return Err(format!("invalid or duplicate SCSVS group {id}"));
        }
        let mut title = None;
        let mut description = Vec::new();
        index += 1;
        while index < lines.len() && !lines[index].starts_with("- gid: ") {
            if let Some(value) = lines[index].strip_prefix("  title: ") {
                title = Some(value.trim().trim_matches('\'').trim().to_owned());
            } else if let Some(value) = lines[index].strip_prefix("  description: ") {
                description.push(value.trim());
                index += 1;
                while index < lines.len() {
                    let line = lines[index];
                    if let Some(value) = line.strip_prefix("    ") {
                        description.push(value.trim());
                        index += 1;
                    } else {
                        break;
                    }
                }
                continue;
            }
            index += 1;
        }
        let title = title
            .filter(|title| !title.is_empty())
            .ok_or_else(|| format!("SCSVS group {id} requires a title"))?;
        groups.push(ScsvsGroup {
            id: id.to_owned(),
            title,
            description: description.join(" "),
        });
    }
    if groups.is_empty() {
        return Err("SCSVS source contains no groups".into());
    }
    Ok(groups)
}

fn semantic_summary(title: &str, text: &str) -> String {
    const MAX_DESCRIPTION_CHARS: usize = 160;

    let mut in_description = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "## Description" {
            in_description = true;
            continue;
        }
        if in_description && trimmed.starts_with("## ") {
            break;
        }
        if !in_description {
            continue;
        }
        if trimmed.is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        if lines.is_empty()
            && (trimmed.starts_with('-')
                || trimmed.starts_with('#')
                || trimmed.starts_with("```")
                || trimmed.starts_with('!'))
        {
            continue;
        }
        lines.push(trimmed);
    }
    let description = lines.join(" ");
    if description.is_empty() {
        return title.to_owned();
    }
    let mut end = description.len();
    for (count, (offset, _)) in description.char_indices().enumerate() {
        if count == MAX_DESCRIPTION_CHARS {
            end = offset;
            break;
        }
    }
    let shortened = if end < description.len() {
        let prefix = &description[..end];
        let boundary = prefix.rfind(char::is_whitespace).unwrap_or(prefix.len());
        format!("{}…", prefix[..boundary].trim_end())
    } else {
        description
    };
    format!("{title}: {shortened}")
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

fn inline_list_item<'a>(metadata: &'a str, key: &str) -> Result<&'a str, String> {
    let prefix = format!("{key}:");
    let value = metadata
        .lines()
        .find_map(|line| line.trim_start().strip_prefix(&prefix).map(str::trim))
        .ok_or_else(|| format!("SCWE frontmatter requires {key}"))?;
    let item = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .map(str::trim)
        .filter(|value| !value.is_empty() && !value.contains(','))
        .ok_or_else(|| format!("SCWE frontmatter {key} must contain exactly one item"))?;
    if !item.starts_with("SCSVS-")
        || !item
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte == b'-')
    {
        return Err(format!(
            "SCWE frontmatter {key} has invalid category {item}"
        ));
    }
    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::{frontmatter, inline_list_item, scalar, semantic_summary};

    #[test]
    fn parses_lf_and_crlf_frontmatter_without_changing_source_text() {
        for text in [
            "---\nid: SCWE-001\ntitle: Example\nmappings:\n  scsvs-cg: [SCSVS-ARCH]\n---\n\n## Description\nBody\n",
            "---\r\nid: SCWE-001\r\ntitle: Example\r\nmappings:\r\n  scsvs-cg: [SCSVS-ARCH]\r\n---\r\n\r\nBody\r\n",
        ] {
            let metadata = frontmatter(text).unwrap();
            assert_eq!(scalar(metadata, "id").unwrap(), "SCWE-001");
            assert_eq!(scalar(metadata, "title").unwrap(), "Example");
            assert_eq!(
                inline_list_item(metadata, "scsvs-cg").unwrap(),
                "SCSVS-ARCH"
            );
            assert!(text.ends_with(if text.contains("\r\n") { "\r\n" } else { "\n" }));
        }
    }

    #[test]
    fn extracts_a_bounded_description_without_splitting_unicode() {
        let text = format!(
            "---\nid: SCWE-001\ntitle: Example\nmappings:\n  scsvs-cg: [SCSVS-CODE]\n---\n\n## Description\n{} block.number and (e.g. identifiers) {}\n\n## Remediation\nOther.\n",
            "é".repeat(90),
            "tail ".repeat(30)
        );
        let summary = semantic_summary("Example", &text);
        assert!(summary.starts_with("Example: "));
        assert!(summary.ends_with('…'));
        assert!(summary.contains("block.number and (e.g. identifiers)"));
        assert!(summary.chars().count() <= "Example: ".chars().count() + 161);
        assert!(!summary.contains("Other"));
    }

    #[test]
    fn requires_one_explicit_scsvs_category() {
        for metadata in [
            "mappings:\n  cwe: [1]",
            "mappings:\n  scsvs-cg: []",
            "mappings:\n  scsvs-cg: [SCSVS-ARCH, SCSVS-CODE]",
            "mappings:\n  scsvs-cg: [scsvs-code]",
        ] {
            assert!(inline_list_item(metadata, "scsvs-cg").is_err());
        }
    }
}
