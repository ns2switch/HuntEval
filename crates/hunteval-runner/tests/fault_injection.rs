use hunteval_resilience::{FaultEvent, FaultKind, FaultSchedule, recover};
use hunteval_runner::FaultController;

#[test]
fn fault_injection_occurs_once_at_each_logical_boundary() {
    let schedule = FaultSchedule {
        seed: 1,
        events: vec![
            FaultEvent {
                logical_sequence: 3,
                kind: FaultKind::UnavailableAgent,
                attempt: 1,
            },
            FaultEvent {
                logical_sequence: 3,
                kind: FaultKind::WorkerFailure,
                attempt: 2,
            },
        ],
    };
    let mut controller = FaultController::new(schedule);
    assert!(controller.at_boundary(2).is_empty());
    assert_eq!(controller.at_boundary(3).len(), 2);
    assert!(controller.at_boundary(3).is_empty());
}

#[test]
fn every_fault_has_a_deterministic_graceful_outcome() {
    let kinds = [
        FaultKind::AgentTimeout,
        FaultKind::MalformedResponse,
        FaultKind::WorkerFailure,
        FaultKind::UnavailableAgent,
        FaultKind::NoisyAgent,
    ];
    for (index, kind) in kinds.into_iter().enumerate() {
        let event = FaultEvent {
            logical_sequence: index as u64 + 1,
            kind,
            attempt: 1,
        };
        let outcome = recover(&event, 1, 1);
        assert!(outcome.recovered, "fault was not contained: {kind:?}");
    }
}

#[test]
fn retry_and_reassignment_budgets_are_enforced() {
    let timeout = FaultEvent {
        logical_sequence: 1,
        kind: FaultKind::AgentTimeout,
        attempt: 1,
    };
    assert!(!recover(&timeout, 0, 1).recovered);
    assert_eq!(recover(&timeout, 2, 1).retries_used, 1);

    let unavailable = FaultEvent {
        logical_sequence: 2,
        kind: FaultKind::UnavailableAgent,
        attempt: 1,
    };
    assert!(!recover(&unavailable, 1, 0).recovered);
    assert!(recover(&unavailable, 0, 1).reassigned);
}
