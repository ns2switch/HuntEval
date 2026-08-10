use hunteval_domain::{SchemaVersion, Sha256Digest, UtcTimestamp};
use serde::{Deserialize, Serialize};

use crate::{AnalyticalResult, CorpusScope, KnowledgeError};

const VERSION: SchemaVersion = SchemaVersion::new(0, 9);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalAuditEvent {
    pub schema_version: SchemaVersion,
    pub sequence: u64,
    pub recorded_at: UtcTimestamp,
    pub scope: CorpusScope,
    pub query_sha256: Sha256Digest,
    pub result_sha256: Sha256Digest,
    pub index_sha256: Sha256Digest,
    pub latency_ms: u64,
    pub cost_microunits: Option<u64>,
    pub previous_event_sha256: Option<Sha256Digest>,
}

#[derive(Debug, Default, Clone)]
pub struct RetrievalAuditJournal {
    events: Vec<RetrievalAuditEvent>,
}

impl RetrievalAuditJournal {
    pub fn append(
        &mut self,
        recorded_at: UtcTimestamp,
        scope: CorpusScope,
        result: &AnalyticalResult,
        latency_ms: u64,
        cost_microunits: Option<u64>,
    ) -> Result<&RetrievalAuditEvent, KnowledgeError> {
        let previous_event_sha256 = self.events.last().map(event_digest).transpose()?;
        self.events.push(RetrievalAuditEvent {
            schema_version: VERSION,
            sequence: self.events.len() as u64 + 1,
            recorded_at,
            scope,
            query_sha256: result.query_sha256,
            result_sha256: Sha256Digest::from_bytes(
                serde_json::to_vec(result).map_err(|_| KnowledgeError::InvalidAuditJournal)?,
            ),
            index_sha256: result.index_sha256,
            latency_ms,
            cost_microunits,
            previous_event_sha256,
        });
        self.events
            .last()
            .ok_or(KnowledgeError::InvalidAuditJournal)
    }

    pub fn replay(events: Vec<RetrievalAuditEvent>) -> Result<Self, KnowledgeError> {
        let mut previous = None;
        for (index, event) in events.iter().enumerate() {
            if event.schema_version != VERSION
                || event.sequence != index as u64 + 1
                || event.previous_event_sha256 != previous
            {
                return Err(KnowledgeError::InvalidAuditJournal);
            }
            previous = Some(event_digest(event)?);
        }
        Ok(Self { events })
    }

    #[must_use]
    pub fn events(&self) -> &[RetrievalAuditEvent] {
        &self.events
    }
}

fn event_digest(event: &RetrievalAuditEvent) -> Result<Sha256Digest, KnowledgeError> {
    let bytes = serde_json::to_vec(event).map_err(|_| KnowledgeError::InvalidAuditJournal)?;
    Ok(Sha256Digest::from_bytes(bytes))
}
