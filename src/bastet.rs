//! Deterministic import of a pinned Bastet label CSV.

use crate::{Corpus, Edge, Kind, Node, Relation, ReviewStatus, Source};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::Path,
};

const LICENSE: &str = "CC-BY-NC-4.0";
const SOURCE_ID: &str = "bastet:dataset";

#[derive(Deserialize)]
struct Row {
    #[serde(rename = "Property")]
    property: String,
    repo_path: String,
    severity: String,
    tag: String,
    subtag: String,
    detail: String,
    description: String,
}

/// Import labeled findings and conservative tag-to-subtag relationships.
///
/// A subtag is connected to a tag only when their association occurs in a row
/// with exactly one tag. Multi-tag rows retain all labels on the finding without
/// inventing a pairing between their independent tag and subtag lists.
pub fn import_bastet(
    path: &Path,
    expected_sha256: &str,
    source_url: &str,
) -> Result<Corpus, String> {
    validate_digest(expected_sha256)?;
    if !source_url.starts_with("https://") {
        return Err("Bastet source URL must use HTTPS".into());
    }
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let actual_sha256 = format!("{:x}", Sha256::digest(&bytes));
    if !actual_sha256.eq_ignore_ascii_case(expected_sha256) {
        return Err(format!(
            "Bastet SHA-256 mismatch: expected {expected_sha256}, got {actual_sha256}"
        ));
    }

    let mut reader = csv::ReaderBuilder::new().from_reader(bytes.as_slice());
    let mut rows = Vec::new();
    let mut properties = HashSet::new();
    let mut tags = BTreeMap::new();
    let mut subtags = BTreeMap::new();
    let mut specializations = BTreeSet::new();
    for result in reader.deserialize::<Row>() {
        let row = result.map_err(|error| error.to_string())?;
        let row_tags = labels(&row.tag);
        if row_tags.is_empty() {
            continue;
        }
        let property = row.property.trim();
        if property.is_empty() || !property.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("labeled Bastet rows require a numeric Property".into());
        }
        if !properties.insert(property.to_owned()) {
            return Err(format!("duplicate Bastet Property: {property}"));
        }
        let row_subtags = labels(&row.subtag);
        for tag in &row_tags {
            insert_label(&mut tags, "tag", tag)?;
        }
        for subtag in &row_subtags {
            insert_label(&mut subtags, "subtag", subtag)?;
        }
        if let [tag] = row_tags.as_slice() {
            for subtag in &row_subtags {
                specializations.insert((label_id("subtag", subtag)?, label_id("tag", tag)?));
            }
        }
        rows.push((row, row_tags, row_subtags));
    }
    if rows.is_empty() {
        return Err("Bastet CSV contains no labeled rows".into());
    }

    let mut nodes = Vec::with_capacity(tags.len() + subtags.len() + rows.len());
    for (id, label) in &tags {
        nodes.push(class_node(id, label, "tag"));
    }
    for (id, label) in &subtags {
        nodes.push(class_node(id, label, "subtag"));
    }

    let mut edges = specializations
        .into_iter()
        .map(|(from, to)| Edge {
            from,
            relation: Relation::Specializes,
            to,
        })
        .collect::<Vec<_>>();
    for (row, row_tags, row_subtags) in rows {
        let property = row.property.trim();
        let severity = row.severity.trim().to_ascii_lowercase();
        if !matches!(severity.as_str(), "high" | "medium" | "low") {
            return Err(format!(
                "labeled Bastet row {property} has unsupported severity"
            ));
        }
        let id = format!("bastet:finding:{property}");
        let mut facets = vec![
            "dataset:bastet".into(),
            format!("severity:{severity}"),
            format!("project:{}", slug(row.repo_path.trim())?),
        ];
        facets.extend(
            row_tags
                .iter()
                .map(|label| Ok(format!("tag:{}", slug(label)?)))
                .collect::<Result<Vec<_>, String>>()?,
        );
        facets.extend(
            row_subtags
                .iter()
                .map(|label| Ok(format!("subtag:{}", slug(label)?)))
                .collect::<Result<Vec<_>, String>>()?,
        );
        let targets = if row_subtags.is_empty() {
            row_tags
                .iter()
                .map(|label| label_id("tag", label))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            row_subtags
                .iter()
                .map(|label| label_id("subtag", label))
                .collect::<Result<Vec<_>, _>>()?
        };
        edges.extend(targets.into_iter().map(|to| Edge {
            from: id.clone(),
            relation: Relation::InstanceOf,
            to,
        }));
        let fallback = format!(
            "{} finding in {}: {}",
            capitalize(&severity),
            row.repo_path.trim(),
            row_subtags.join(", ")
        );
        nodes.push(Node {
            id,
            kind: Kind::Finding,
            summary: bounded_summary(&row.description, &fallback),
            definition: row.description,
            facets,
            exclusions: Vec::new(),
            sources: vec![SOURCE_ID.into()],
            applicability: Vec::new(),
            mappings: [
                format!("bastet:property:{property}"),
                format!("bastet:repo:{}", row.repo_path.trim()),
                format!("bastet:report:{}", row.detail.trim()),
            ]
            .into_iter()
            .filter(|mapping| !mapping.ends_with(':'))
            .collect(),
            review: ReviewStatus::Imported,
            code: Vec::new(),
        });
    }
    nodes.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    edges.sort_unstable_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then(relation_rank(left.relation).cmp(&relation_rank(right.relation)))
            .then(left.to.cmp(&right.to))
    });

    Ok(Corpus {
        revision: format!("bastet-{}-source-v1", &actual_sha256[..12]),
        sources: vec![Source {
            id: SOURCE_ID.into(),
            title: "Bastet labeled DeFi findings".into(),
            url: source_url.into(),
            revision: actual_sha256,
            license: LICENSE.into(),
        }],
        nodes,
        edges,
    })
}

fn validate_digest(digest: &str) -> Result<(), String> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Bastet revision must be a 64-character SHA-256 digest".into());
    }
    Ok(())
}

fn labels(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn insert_label(
    labels: &mut BTreeMap<String, String>,
    layer: &str,
    label: &str,
) -> Result<(), String> {
    let id = label_id(layer, label)?;
    if let Some(previous) = labels.insert(id.clone(), label.to_owned())
        && previous != label
    {
        return Err(format!(
            "Bastet labels collide after normalization: {previous:?} and {label:?} use {id:?}"
        ));
    }
    Ok(())
}

fn label_id(layer: &str, label: &str) -> Result<String, String> {
    Ok(format!("bastet:{layer}:{}", slug(label)?))
}

fn slug(value: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            if separator && !output.is_empty() {
                output.push('-');
            }
            output.push(character.to_ascii_lowercase());
            separator = false;
        } else {
            separator = true;
        }
    }
    if output.is_empty() {
        return Err(format!("cannot derive a stable ID from {value:?}"));
    }
    Ok(output)
}

fn class_node(id: &str, label: &str, layer: &str) -> Node {
    Node {
        id: id.into(),
        kind: Kind::FailureMode,
        summary: label.into(),
        definition: String::new(),
        facets: vec!["dataset:bastet".into(), format!("level:{layer}")],
        exclusions: Vec::new(),
        sources: vec![SOURCE_ID.into()],
        applicability: Vec::new(),
        mappings: Vec::new(),
        review: ReviewStatus::Imported,
        code: Vec::new(),
    }
}

fn bounded_summary(description: &str, fallback: &str) -> String {
    const MAX_CHARS: usize = 160;

    let compact = description.split_whitespace().collect::<Vec<_>>().join(" ");
    let text = if compact.is_empty() {
        fallback
    } else {
        &compact
    };
    if text.chars().count() <= MAX_CHARS {
        return text.to_owned();
    }
    let mut end = text.len();
    for (count, (offset, _)) in text.char_indices().enumerate() {
        if count == MAX_CHARS {
            end = offset;
            break;
        }
    }
    let prefix = &text[..end];
    let boundary = prefix.rfind(char::is_whitespace).unwrap_or(prefix.len());
    format!("{}…", prefix[..boundary].trim_end())
}

fn capitalize(value: &str) -> String {
    let mut characters = value.chars();
    characters
        .next()
        .map(|first| first.to_uppercase().chain(characters).collect())
        .unwrap_or_default()
}

fn relation_rank(relation: Relation) -> u8 {
    match relation {
        Relation::Specializes => 0,
        Relation::Violates => 1,
        Relation::InstanceOf => 2,
        Relation::RelatedTo => 3,
    }
}
