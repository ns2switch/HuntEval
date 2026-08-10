use std::collections::BTreeMap;

use hunteval_domain::{SchemaVersion, Sha256Digest};
use hunteval_knowledge::{
    AnalyticalCorpusManifest, AnalyticalIndex, AnalyticalQuery, AnalyticalSourceKind, CorpusScope,
    CorpusSource, RetrievalAuditJournal, VerifiedAnalyticalDocument,
};

fn source(id: &str, kind: AnalyticalSourceKind) -> CorpusSource {
    CorpusSource {
        id: id.to_owned(),
        kind,
        path: format!("{id}.json"),
        artifact_sha256: Sha256Digest::from_bytes(id),
        verified: true,
    }
}

#[test]
fn retrieval_audit_is_hash_linked_and_replayable() -> Result<(), Box<dyn std::error::Error>> {
    let run = source("run-01", AnalyticalSourceKind::Run);
    let corpus = AnalyticalCorpusManifest {
        schema_version: SchemaVersion::new(0, 9),
        id: "history".to_owned(),
        scope: CorpusScope::EvaluatorAnalytics,
        sources: vec![run.clone()],
    };
    let index = AnalyticalIndex::build(
        &corpus,
        vec![VerifiedAnalyticalDocument {
            source: run,
            fields: BTreeMap::from([("finding".to_owned(), "access key".to_owned())]),
        }],
    )?;
    let result = index.query(&AnalyticalQuery {
        schema_version: SchemaVersion::new(0, 9),
        index_sha256: index.manifest().index_sha256,
        scope: CorpusScope::EvaluatorAnalytics,
        terms: vec!["access".to_owned()],
        source_kinds: None,
        max_results: 1,
    })?;
    let mut journal = RetrievalAuditJournal::default();
    journal.append(
        "2026-08-10T00:00:00Z".parse()?,
        CorpusScope::EvaluatorAnalytics,
        &result,
        3,
        None,
    )?;
    journal.append(
        "2026-08-10T00:00:01Z".parse()?,
        CorpusScope::EvaluatorAnalytics,
        &result,
        2,
        None,
    )?;
    let replayed = RetrievalAuditJournal::replay(journal.events().to_vec())?;
    assert_eq!(replayed.events().len(), 2);
    let mut tampered = replayed.events().to_vec();
    tampered[1].sequence = 8;
    assert!(RetrievalAuditJournal::replay(tampered).is_err());
    Ok(())
}

#[test]
fn verified_artifacts_are_searchable_with_exact_citations() -> Result<(), Box<dyn std::error::Error>>
{
    let run = source("run-01", AnalyticalSourceKind::Run);
    let corpus = AnalyticalCorpusManifest {
        schema_version: SchemaVersion::new(0, 9),
        id: "history".to_owned(),
        scope: CorpusScope::EvaluatorAnalytics,
        sources: vec![run.clone()],
    };
    let index = AnalyticalIndex::build(
        &corpus,
        vec![VerifiedAnalyticalDocument {
            source: run.clone(),
            fields: BTreeMap::from([(
                "finding".to_owned(),
                "Compromised identity used an access key".to_owned(),
            )]),
        }],
    )?;
    let result = index.query(&AnalyticalQuery {
        schema_version: SchemaVersion::new(0, 9),
        index_sha256: index.manifest().index_sha256,
        scope: CorpusScope::EvaluatorAnalytics,
        terms: vec!["access key".to_owned()],
        source_kinds: None,
        max_results: 10,
    })?;
    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].artifact_sha256, run.artifact_sha256);
    Ok(())
}

#[test]
fn deployment_corpus_rejects_evaluator_artifacts() {
    let corpus = AnalyticalCorpusManifest {
        schema_version: SchemaVersion::new(0, 9),
        id: "unsafe".to_owned(),
        scope: CorpusScope::DeploymentVisible,
        sources: vec![source("run-01", AnalyticalSourceKind::Run)],
    };
    assert!(corpus.validate().is_err());
}
