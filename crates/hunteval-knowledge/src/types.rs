use std::{fmt, path::Path};

use hunteval_domain::SchemaVersion;
use serde::{Deserialize, Deserializer, Serialize, de};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct DocumentId(String);

impl DocumentId {
    pub fn new(value: impl Into<String>) -> Result<Self, KnowledgeError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || !value
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            || !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
            })
        {
            return Err(KnowledgeError::InvalidDocumentId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for DocumentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeDocument {
    pub id: DocumentId,
    pub title: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeManifest {
    pub schema_version: SchemaVersion,
    #[serde(default)]
    pub enabled: bool,
    pub corpus_root: String,
    pub documents: Vec<KnowledgeDocument>,
    pub max_documents: u32,
    pub max_query_tokens: u32,
    pub max_output_tokens: u32,
}

impl KnowledgeManifest {
    pub fn validate(&self) -> Result<(), KnowledgeError> {
        if !safe_relative(&self.corpus_root)
            || self.max_documents == 0
            || self.max_query_tokens == 0
            || self.max_output_tokens == 0
            || self
                .documents
                .iter()
                .any(|document| document.title.trim().is_empty() || !safe_relative(&document.path))
        {
            return Err(KnowledgeError::InvalidManifest);
        }
        let unique: std::collections::BTreeSet<_> =
            self.documents.iter().map(|document| &document.id).collect();
        if unique.len() != self.documents.len() {
            return Err(KnowledgeError::InvalidManifest);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalRequest {
    pub query: String,
    pub max_documents: u32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievedDocument {
    pub id: DocumentId,
    pub title: String,
    pub content: String,
    pub untrusted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    pub document_id: DocumentId,
    pub start_byte: usize,
    pub end_byte: usize,
    pub quote: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalResult {
    pub query: String,
    pub documents: Vec<RetrievedDocument>,
    pub citations: Vec<Citation>,
    pub latency_ms: u64,
    pub cost_microunits: u64,
}

pub trait LocalIndex: fmt::Debug {
    fn retrieve(&self, request: &RetrievalRequest) -> Result<RetrievalResult, KnowledgeError>;
}

fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && !Path::new(value).components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
}

#[derive(Debug, Error)]
pub enum KnowledgeError {
    #[error("knowledge document identifier is invalid")]
    InvalidDocumentId,
    #[error("knowledge manifest or corpus path is invalid")]
    InvalidManifest,
    #[error("knowledge retrieval is disabled")]
    Disabled,
    #[error("retrieval request exceeds a configured budget")]
    BudgetExceeded,
    #[error("knowledge document is unavailable or outside the corpus")]
    DocumentUnavailable,
    #[error("knowledge document is not valid UTF-8")]
    InvalidEncoding,
}
