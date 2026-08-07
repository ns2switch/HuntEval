use hunteval_knowledge::{
    KnowledgeError, KnowledgeManifest, LocalIndex, RetrievalRequest, RetrievalResult,
};
use thiserror::Error;

#[derive(Debug)]
pub struct KnowledgeController<I> {
    manifest: KnowledgeManifest,
    index: Option<I>,
    records: Vec<RetrievalResult>,
}

impl<I: LocalIndex> KnowledgeController<I> {
    #[must_use]
    pub fn new(manifest: KnowledgeManifest, index: Option<I>) -> Self {
        Self {
            manifest,
            index,
            records: Vec::new(),
        }
    }

    pub fn retrieve(
        &mut self,
        request: &RetrievalRequest,
    ) -> Result<&RetrievalResult, KnowledgeControllerError> {
        if !self.manifest.enabled {
            return Err(KnowledgeControllerError::Disabled);
        }
        let index = self
            .index
            .as_ref()
            .ok_or(KnowledgeControllerError::Unavailable)?;
        let result = index
            .retrieve(request)
            .map_err(KnowledgeControllerError::Knowledge)?;
        self.records.push(result);
        self.records
            .last()
            .ok_or(KnowledgeControllerError::Unavailable)
    }

    #[must_use]
    pub fn records(&self) -> &[RetrievalResult] {
        &self.records
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeControllerError {
    #[error("knowledge retrieval is disabled")]
    Disabled,
    #[error("local knowledge index is unavailable")]
    Unavailable,
    #[error("knowledge retrieval failed: {0}")]
    Knowledge(KnowledgeError),
}
