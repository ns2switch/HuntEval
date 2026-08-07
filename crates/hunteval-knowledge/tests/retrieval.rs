use std::fs;

use hunteval_domain::SchemaVersion;
use hunteval_knowledge::{
    DocumentId, KnowledgeDocument, KnowledgeManifest, LocalIndex, LocalKnowledgeIndex,
    RetrievalRequest,
};

fn manifest() -> Result<KnowledgeManifest, Box<dyn std::error::Error>> {
    Ok(KnowledgeManifest {
        schema_version: SchemaVersion::new(0, 1),
        enabled: true,
        corpus_root: "corpus".into(),
        documents: vec![
            KnowledgeDocument {
                id: DocumentId::new("identity")?,
                title: "Identity overview".into(),
                path: "identity.txt".into(),
            },
            KnowledgeDocument {
                id: DocumentId::new("network")?,
                title: "Network overview".into(),
                path: "network.txt".into(),
            },
        ],
        max_documents: 2,
        max_query_tokens: 8,
        max_output_tokens: 64,
    })
}

#[test]
fn local_retrieval_is_deterministic_cited_and_budgeted() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    fs::create_dir(root.path().join("corpus"))?;
    fs::write(
        root.path().join("corpus/identity.txt"),
        "Identity administrators approve privileged role changes.",
    )?;
    fs::write(
        root.path().join("corpus/network.txt"),
        "Network owners approve firewall changes.",
    )?;
    let index = LocalKnowledgeIndex::open(root.path(), manifest()?)?;
    let request = RetrievalRequest {
        query: "identity privileged".into(),
        max_documents: 1,
        max_tokens: 16,
    };
    let first = index.retrieve(&request)?;
    assert_eq!(first, index.retrieve(&request)?);
    assert_eq!(first.documents[0].id.as_str(), "identity");
    let citation = &first.citations[0];
    assert_eq!(
        citation.quote,
        first.documents[0].content[citation.start_byte..citation.end_byte]
    );

    let over_budget = RetrievalRequest {
        max_documents: 3,
        ..request
    };
    assert!(index.retrieve(&over_budget).is_err());
    Ok(())
}
