use hunteval_domain::SchemaVersion;
use hunteval_knowledge::{
    KnowledgeError, KnowledgeManifest, LocalIndex, RetrievalRequest, RetrievalResult,
};
use hunteval_runner::KnowledgeController;

#[derive(Debug)]
struct UnusedIndex;

impl LocalIndex for UnusedIndex {
    fn retrieve(&self, _request: &RetrievalRequest) -> Result<RetrievalResult, KnowledgeError> {
        Err(KnowledgeError::Disabled)
    }
}

#[test]
fn benchmark_operation_has_no_retrieval_by_default() {
    let manifest = KnowledgeManifest {
        schema_version: SchemaVersion::new(0, 1),
        enabled: false,
        corpus_root: ".".into(),
        documents: Vec::new(),
        max_documents: 1,
        max_query_tokens: 1,
        max_output_tokens: 1,
    };
    let mut controller = KnowledgeController::new(manifest, Some(UnusedIndex));
    let request = RetrievalRequest {
        query: "identity".into(),
        max_documents: 1,
        max_tokens: 1,
    };
    assert!(controller.retrieve(&request).is_err());
    assert!(controller.records().is_empty());
}
