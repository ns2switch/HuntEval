//! Optional, deterministic, network-free retrieval over author-provided documents.

mod index;
mod types;

pub use index::LocalKnowledgeIndex;
pub use types::{
    Citation, DocumentId, KnowledgeDocument, KnowledgeError, KnowledgeManifest, LocalIndex,
    RetrievalRequest, RetrievalResult, RetrievedDocument,
};
