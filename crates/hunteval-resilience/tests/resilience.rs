use hunteval_resilience::{FaultKind, FaultProfile, graceful_degradation, schedule};

#[test]
fn schedule_reproduces_from_seed_at_logical_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let profile = FaultProfile {
        id: "agent-failures".into(),
        seed: 41,
        faults: vec![
            FaultKind::AgentTimeout,
            FaultKind::WorkerFailure,
            FaultKind::MalformedResponse,
        ],
        retry_budget: 2,
    };
    assert_eq!(schedule(&profile, 12)?, schedule(&profile, 12)?);
    assert!(
        schedule(&profile, 12)?
            .events
            .iter()
            .all(|event| (1..=12).contains(&event.logical_sequence))
    );
    Ok(())
}

#[test]
fn degradation_has_documented_zero_denominator_behavior() -> Result<(), Box<dyn std::error::Error>>
{
    assert_eq!(graceful_degradation(0.8, 0.4)?.value, Some(0.5));
    assert_eq!(graceful_degradation(0.0, 0.0)?.value, None);
    assert!(graceful_degradation(1.1, 0.5).is_err());
    Ok(())
}
