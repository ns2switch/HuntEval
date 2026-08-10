//! Optional, deterministic, network-free retrieval over author-provided documents.

mod analytical;
mod audit;
mod index;
mod types;

pub use analytical::{
    AnalyticalCorpusBundle, AnalyticalCorpusManifest, AnalyticalIndex, AnalyticalIndexManifest,
    AnalyticalMatch, AnalyticalQuery, AnalyticalResult, AnalyticalSourceKind, CorpusScope,
    CorpusSource, VerifiedAnalyticalDocument,
};
pub use audit::{RetrievalAuditEvent, RetrievalAuditJournal};
pub use index::LocalKnowledgeIndex;
pub use types::{
    Citation, DocumentId, KnowledgeDocument, KnowledgeError, KnowledgeManifest, LocalIndex,
    RetrievalRequest, RetrievalResult, RetrievedDocument,
};
