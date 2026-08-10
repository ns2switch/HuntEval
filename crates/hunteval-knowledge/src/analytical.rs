use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{SchemaVersion, Sha256Digest};
use serde::{Deserialize, Serialize};

use crate::KnowledgeError;

const VERSION: SchemaVersion = SchemaVersion::new(0, 9);
const MAX_SOURCES: usize = 10_000;
const MAX_FIELDS: usize = 128;
const MAX_FIELD_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusScope {
    EvaluatorAnalytics,
    DeploymentVisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticalSourceKind {
    Run,
    Benchmark,
    Report,
    Topology,
    Diagnosis,
    Improvement,
    Document,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusSource {
    pub id: String,
    pub kind: AnalyticalSourceKind,
    pub path: String,
    pub artifact_sha256: Sha256Digest,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalCorpusManifest {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub scope: CorpusScope,
    pub sources: Vec<CorpusSource>,
}

impl AnalyticalCorpusManifest {
    pub fn validate(&self) -> Result<(), KnowledgeError> {
        if self.schema_version != VERSION
            || !safe_id(&self.id)
            || self.sources.is_empty()
            || self.sources.len() > MAX_SOURCES
            || self.sources.iter().any(|source| {
                !source.verified
                    || !safe_id(&source.id)
                    || !crate::types::safe_relative(&source.path)
                    || (self.scope == CorpusScope::DeploymentVisible
                        && source.kind != AnalyticalSourceKind::Document)
            })
        {
            return Err(KnowledgeError::InvalidAnalyticalCorpus);
        }
        let identities: BTreeSet<_> = self.sources.iter().map(|source| &source.id).collect();
        if identities.len() != self.sources.len() {
            return Err(KnowledgeError::InvalidAnalyticalCorpus);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedAnalyticalDocument {
    pub source: CorpusSource,
    pub fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalCorpusBundle {
    pub corpus: AnalyticalCorpusManifest,
    pub documents: Vec<VerifiedAnalyticalDocument>,
}

impl AnalyticalCorpusBundle {
    pub fn open(&self) -> Result<AnalyticalIndex, KnowledgeError> {
        AnalyticalIndex::build(&self.corpus, self.documents.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalIndexManifest {
    pub schema_version: SchemaVersion,
    pub corpus_sha256: Sha256Digest,
    pub source_hashes: Vec<Sha256Digest>,
    pub index_sha256: Sha256Digest,
}

#[derive(Debug, Clone)]
pub struct AnalyticalIndex {
    manifest: AnalyticalIndexManifest,
    scope: CorpusScope,
    documents: Vec<VerifiedAnalyticalDocument>,
}

impl AnalyticalIndex {
    pub fn build(
        corpus: &AnalyticalCorpusManifest,
        mut documents: Vec<VerifiedAnalyticalDocument>,
    ) -> Result<Self, KnowledgeError> {
        corpus.validate()?;
        documents.sort_by(|left, right| left.source.id.cmp(&right.source.id));
        if documents.len() != corpus.sources.len() {
            return Err(KnowledgeError::AnalyticalSourceMismatch);
        }
        let expected: BTreeMap<_, _> = corpus
            .sources
            .iter()
            .map(|source| (source.id.as_str(), source))
            .collect();
        for document in &documents {
            if expected.get(document.source.id.as_str()) != Some(&&document.source)
                || document.fields.is_empty()
                || document.fields.len() > MAX_FIELDS
                || document
                    .fields
                    .iter()
                    .any(|(name, value)| !safe_id(name) || value.len() > MAX_FIELD_BYTES)
            {
                return Err(KnowledgeError::AnalyticalSourceMismatch);
            }
        }
        let corpus_bytes =
            serde_json::to_vec(corpus).map_err(|_| KnowledgeError::InvalidAnalyticalCorpus)?;
        let index_bytes = canonical_index_bytes(&documents)?;
        let manifest = AnalyticalIndexManifest {
            schema_version: VERSION,
            corpus_sha256: Sha256Digest::from_bytes(corpus_bytes),
            source_hashes: documents
                .iter()
                .map(|document| document.source.artifact_sha256)
                .collect(),
            index_sha256: Sha256Digest::from_bytes(index_bytes),
        };
        Ok(Self {
            manifest,
            scope: corpus.scope,
            documents,
        })
    }

    #[must_use]
    pub const fn manifest(&self) -> &AnalyticalIndexManifest {
        &self.manifest
    }

    pub fn query(&self, query: &AnalyticalQuery) -> Result<AnalyticalResult, KnowledgeError> {
        query.validate()?;
        if query.index_sha256 != self.manifest.index_sha256 || query.scope != self.scope {
            return Err(KnowledgeError::AnalyticalAuthorizationDenied);
        }
        let terms: Vec<String> = query.terms.iter().map(|term| term.to_lowercase()).collect();
        let mut matches = Vec::new();
        for document in &self.documents {
            if query
                .source_kinds
                .as_ref()
                .is_some_and(|kinds| !kinds.contains(&document.source.kind))
            {
                continue;
            }
            for (field, value) in &document.fields {
                let normalized = value.to_lowercase();
                if terms.iter().all(|term| normalized.contains(term)) {
                    matches.push(AnalyticalMatch {
                        source_id: document.source.id.clone(),
                        source_kind: document.source.kind,
                        artifact_sha256: document.source.artifact_sha256,
                        field: field.clone(),
                        excerpt: bounded_excerpt(value, 240),
                    });
                    if matches.len() == query.max_results as usize {
                        break;
                    }
                }
            }
            if matches.len() == query.max_results as usize {
                break;
            }
        }
        Ok(AnalyticalResult {
            schema_version: VERSION,
            query_sha256: query.digest()?,
            index_sha256: self.manifest.index_sha256,
            matches,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalQuery {
    pub schema_version: SchemaVersion,
    pub index_sha256: Sha256Digest,
    pub scope: CorpusScope,
    pub terms: Vec<String>,
    pub source_kinds: Option<BTreeSet<AnalyticalSourceKind>>,
    pub max_results: u32,
}

impl AnalyticalQuery {
    pub fn validate(&self) -> Result<(), KnowledgeError> {
        if self.schema_version != VERSION
            || self.terms.is_empty()
            || self.terms.len() > 16
            || self
                .terms
                .iter()
                .any(|term| term.trim().is_empty() || term.len() > 128)
            || self.max_results == 0
            || self.max_results > 100
        {
            return Err(KnowledgeError::InvalidAnalyticalQuery);
        }
        Ok(())
    }

    fn digest(&self) -> Result<Sha256Digest, KnowledgeError> {
        let bytes = serde_json::to_vec(self).map_err(|_| KnowledgeError::InvalidAnalyticalQuery)?;
        Ok(Sha256Digest::from_bytes(bytes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalMatch {
    pub source_id: String,
    pub source_kind: AnalyticalSourceKind,
    pub artifact_sha256: Sha256Digest,
    pub field: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalResult {
    pub schema_version: SchemaVersion,
    pub query_sha256: Sha256Digest,
    pub index_sha256: Sha256Digest,
    pub matches: Vec<AnalyticalMatch>,
}

fn canonical_index_bytes(
    documents: &[VerifiedAnalyticalDocument],
) -> Result<Vec<u8>, KnowledgeError> {
    let serializable: Vec<_> = documents
        .iter()
        .map(|document| (&document.source, &document.fields))
        .collect();
    serde_json::to_vec(&serializable).map_err(|_| KnowledgeError::InvalidAnalyticalCorpus)
}

fn bounded_excerpt(value: &str, maximum: usize) -> String {
    if value.len() <= maximum {
        return value.to_owned();
    }
    let mut end = maximum;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

fn safe_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}
