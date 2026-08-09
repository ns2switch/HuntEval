use hunteval_statistics::{CalibrationObservation, CalibrationStatus, evaluate_calibration};

#[test]
fn calibration_is_deterministic_and_bounded() -> Result<(), Box<dyn std::error::Error>> {
    let observations = [
        CalibrationObservation {
            confidence: 0.9,
            outcome: true,
            predicted_severity: Some(3),
            actual_severity: Some(3),
        },
        CalibrationObservation {
            confidence: 0.2,
            outcome: false,
            predicted_severity: Some(2),
            actual_severity: Some(1),
        },
    ];
    let result = evaluate_calibration(&observations)?;
    assert_eq!(result.status, CalibrationStatus::Applicable);
    assert_eq!(result.sample_count, 2);
    assert_eq!(result.severity_accuracy, Some(0.5));
    assert!(result.brier_score.is_some_and(|value| value < 0.03));
    assert_eq!(result, evaluate_calibration(&observations)?);
    Ok(())
}

#[test]
fn calibration_never_infers_missing_private_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let unavailable = evaluate_calibration(&[])?;
    assert_eq!(unavailable.status, CalibrationStatus::Unavailable);
    assert_eq!(unavailable.brier_score, None);

    let malformed = [CalibrationObservation {
        confidence: 1.1,
        outcome: true,
        predicted_severity: None,
        actual_severity: None,
    }];
    assert!(evaluate_calibration(&malformed).is_err());
    Ok(())
}
