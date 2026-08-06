use std::collections::BTreeSet;

use hunteval_domain::{EventId, MessageId};

#[derive(Debug, Clone)]
pub(super) struct ActionRecord {
    pub(super) request_message_id: MessageId,
    pub(super) event_ids: Option<BTreeSet<EventId>>,
}
