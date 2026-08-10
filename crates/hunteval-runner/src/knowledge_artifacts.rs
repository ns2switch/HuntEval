use std::{collections::BTreeMap, fs, io::Read, path::Path};

use hunteval_knowledge::{
    AnalyticalCorpusBundle, AnalyticalCorpusManifest, AnalyticalIndexManifest, AnalyticalQuery,
    AnalyticalResult, VerifiedAnalyticalDocument,
};
use serde_json::Value;
use thiserror::Error;

use crate::ReportFormat;

const MAX_SOURCE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_FIELDS: usize = 128;

pub fn validate_analytical_manifest(
    bytes: &[u8],
) -> Result<AnalyticalCorpusManifest, AnalyticalCorpusLoadError> {
    let manifest: AnalyticalCorpusManifest =
        serde_json::from_slice(bytes).map_err(|_| AnalyticalCorpusLoadError::InvalidManifest)?;
    manifest
        .validate()
        .map_err(AnalyticalCorpusLoadError::Knowledge)?;
    Ok(manifest)
}

pub fn build_analytical_index(
    root: &Path,
    manifest_bytes: &[u8],
) -> Result<AnalyticalIndexManifest, AnalyticalCorpusLoadError> {
    let bundle = load_analytical_corpus(root, validate_analytical_manifest(manifest_bytes)?)?;
    Ok(bundle
        .open()
        .map_err(AnalyticalCorpusLoadError::Knowledge)?
        .manifest()
        .clone())
}

pub fn query_analytical_index(
    root: &Path,
    manifest_bytes: &[u8],
    query_bytes: &[u8],
) -> Result<AnalyticalResult, AnalyticalCorpusLoadError> {
    let query: AnalyticalQuery =
        serde_json::from_slice(query_bytes).map_err(|_| AnalyticalCorpusLoadError::InvalidQuery)?;
    let bundle = load_analytical_corpus(root, validate_analytical_manifest(manifest_bytes)?)?;
    bundle
        .open()
        .map_err(AnalyticalCorpusLoadError::Knowledge)?
        .query(&query)
        .map_err(AnalyticalCorpusLoadError::Knowledge)
}

pub fn render_analytical_result(
    result: &AnalyticalResult,
    format: ReportFormat,
) -> Result<Vec<u8>, AnalyticalCorpusLoadError> {
    let report = hunteval_reporting::AnalyticalReport {
        schema_version: result.schema_version.to_string(),
        query_sha256: result.query_sha256.to_string(),
        index_sha256: result.index_sha256.to_string(),
        matches: result
            .matches
            .iter()
            .map(|item| hunteval_reporting::AnalyticalReportMatch {
                source_id: item.source_id.clone(),
                source_kind: serde_json::to_value(item.source_kind)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_owned))
                    .unwrap_or_else(|| "unavailable".to_owned()),
                artifact_sha256: item.artifact_sha256.to_string(),
                field: item.field.clone(),
                excerpt: item.excerpt.clone(),
            })
            .collect(),
        limitations: vec![
            "Results are lexical projections over verified public fields.".to_owned(),
            "No missing metric, causal claim, validation, approval, or transfer claim is inferred."
                .to_owned(),
        ],
    };
    match format {
        ReportFormat::Json => {
            let mut bytes = serde_json::to_vec_pretty(result)
                .map_err(|_| AnalyticalCorpusLoadError::Reporting)?;
            bytes.push(b'\n');
            Ok(bytes)
        }
        ReportFormat::Html => report
            .render_html()
            .map_err(|_| AnalyticalCorpusLoadError::Reporting),
    }
}

pub fn load_analytical_corpus(
    root: &Path,
    manifest: AnalyticalCorpusManifest,
) -> Result<AnalyticalCorpusBundle, AnalyticalCorpusLoadError> {
    manifest
        .validate()
        .map_err(AnalyticalCorpusLoadError::Knowledge)?;
    let root_metadata =
        fs::symlink_metadata(root).map_err(|_| AnalyticalCorpusLoadError::InvalidRoot)?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(AnalyticalCorpusLoadError::InvalidRoot);
    }
    let root = root
        .canonicalize()
        .map_err(|_| AnalyticalCorpusLoadError::InvalidRoot)?;
    let mut documents = Vec::with_capacity(manifest.sources.len());
    for source in &manifest.sources {
        let bytes = read_bounded_source(&root, Path::new(&source.path))?;
        if hunteval_domain::Sha256Digest::from_bytes(&bytes) != source.artifact_sha256 {
            return Err(AnalyticalCorpusLoadError::DigestMismatch);
        }
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|_| AnalyticalCorpusLoadError::InvalidPublicArtifact)?;
        reject_private_fields(&value)?;
        let mut fields = BTreeMap::new();
        flatten_fields("", &value, &mut fields)?;
        if fields.is_empty() || fields.len() > MAX_FIELDS {
            return Err(AnalyticalCorpusLoadError::InvalidPublicArtifact);
        }
        documents.push(VerifiedAnalyticalDocument {
            source: source.clone(),
            fields,
        });
    }
    let bundle = AnalyticalCorpusBundle {
        corpus: manifest,
        documents,
    };
    bundle
        .open()
        .map_err(AnalyticalCorpusLoadError::Knowledge)?;
    Ok(bundle)
}

fn read_bounded_source(root: &Path, relative: &Path) -> Result<Vec<u8>, AnalyticalCorpusLoadError> {
    let mut candidate = root.to_path_buf();
    for component in relative.components() {
        candidate.push(component);
        let metadata = fs::symlink_metadata(&candidate)
            .map_err(|_| AnalyticalCorpusLoadError::InvalidSource)?;
        if metadata.file_type().is_symlink() {
            return Err(AnalyticalCorpusLoadError::InvalidSource);
        }
    }
    let metadata =
        fs::symlink_metadata(&candidate).map_err(|_| AnalyticalCorpusLoadError::InvalidSource)?;
    if !metadata.is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err(AnalyticalCorpusLoadError::InvalidSource);
    }
    #[cfg(unix)]
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file = options
        .open(&candidate)
        .map_err(|_| AnalyticalCorpusLoadError::InvalidSource)?;
    let opened = file
        .metadata()
        .map_err(|_| AnalyticalCorpusLoadError::InvalidSource)?;
    if !opened.is_file() || opened.len() != metadata.len() {
        return Err(AnalyticalCorpusLoadError::InvalidSource);
    }
    #[cfg(unix)]
    if opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
        return Err(AnalyticalCorpusLoadError::InvalidSource);
    }
    let mut bytes = Vec::new();
    file.take(MAX_SOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AnalyticalCorpusLoadError::InvalidSource)?;
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err(AnalyticalCorpusLoadError::InvalidSource);
    }
    Ok(bytes)
}

fn reject_private_fields(value: &Value) -> Result<(), AnalyticalCorpusLoadError> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let normalized = key.to_ascii_lowercase();
                if matches!(
                    normalized.as_str(),
                    "ground_truth"
                        | "hidden_test"
                        | "hidden_test_results"
                        | "reference_query"
                        | "evaluator_only"
                ) {
                    return Err(AnalyticalCorpusLoadError::PrivateField);
                }
                reject_private_fields(child)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                reject_private_fields(child)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn flatten_fields(
    prefix: &str,
    value: &Value,
    output: &mut BTreeMap<String, String>,
) -> Result<(), AnalyticalCorpusLoadError> {
    if output.len() > MAX_FIELDS {
        return Err(AnalyticalCorpusLoadError::InvalidPublicArtifact);
    }
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let field = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                if field.len() <= 128 {
                    flatten_fields(&field, child, output)?;
                }
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                let field = format!("{prefix}.{index}");
                if field.len() <= 128 {
                    flatten_fields(&field, child, output)?;
                }
            }
        }
        Value::String(text) if !prefix.is_empty() && text.len() <= 1_048_576 => {
            output.insert(prefix.to_owned(), text.clone());
        }
        Value::Number(number) if !prefix.is_empty() => {
            output.insert(prefix.to_owned(), number.to_string());
        }
        Value::Bool(boolean) if !prefix.is_empty() => {
            output.insert(prefix.to_owned(), boolean.to_string());
        }
        Value::Null | Value::String(_) | Value::Number(_) | Value::Bool(_) => {}
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum AnalyticalCorpusLoadError {
    #[error("analytical corpus manifest is malformed")]
    InvalidManifest,
    #[error("analytical query is malformed")]
    InvalidQuery,
    #[error("analytical corpus root is unavailable")]
    InvalidRoot,
    #[error("analytical corpus source is not a bounded root-confined regular file")]
    InvalidSource,
    #[error("analytical corpus source digest does not match")]
    DigestMismatch,
    #[error("analytical corpus source is not a supported bounded public artifact")]
    InvalidPublicArtifact,
    #[error("analytical corpus source contains a prohibited private field")]
    PrivateField,
    #[error("analytical result could not be rendered safely")]
    Reporting,
    #[error("analytical corpus contract failed validation: {0}")]
    Knowledge(#[source] hunteval_knowledge::KnowledgeError),
}
