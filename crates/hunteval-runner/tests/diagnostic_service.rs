use std::{collections::BTreeMap, fs, io, path::Path};

use hunteval_domain::{FinalSubmission, RunId, SchemaVersion, Sha256Digest};
use hunteval_protocol::{ProtocolEnvelope, ProtocolPayload, TrajectoryRecorder};
use hunteval_runner::{
    DiagnosticGenerationError, DiagnosticVerificationStatus, RunManifest, generate_run_diagnosis,
    verify_diagnostic_bundle,
};

#[test]
fn verified_run_generates_deterministic_content_addressed_diagnosis()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let run = root.path().join("run");
    fs::create_dir(&run)?;
    write_completed_run(&run)?;
    let first = root.path().join("diagnosis-first");
    let second = root.path().join("diagnosis-second");
    generate_run_diagnosis(&run, &first)?;
    generate_run_diagnosis(&run, &second)?;

    for file in [
        "run-diagnosis.json",
        "bottleneck-observations.json",
        "bottleneck-analysis.json",
        "diagnostic-report.json",
        "diagnostic-report.html",
        "diagnostic-bundle-manifest.json",
    ] {
        assert_eq!(fs::read(first.join(file))?, fs::read(second.join(file))?);
    }
    assert_eq!(
        verify_diagnostic_bundle(&first).status,
        DiagnosticVerificationStatus::Verified
    );
    assert!(matches!(
        generate_run_diagnosis(&run, &first),
        Err(DiagnosticGenerationError::AlreadyExists)
    ));
    Ok(())
}

#[test]
fn verification_detects_tampering_and_generation_rejects_source_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let run = root.path().join("run");
    fs::create_dir(&run)?;
    write_completed_run(&run)?;
    let output = root.path().join("diagnosis");
    generate_run_diagnosis(&run, &output)?;
    fs::write(output.join("run-diagnosis.json"), b"{}\n")?;
    assert_eq!(
        verify_diagnostic_bundle(&output).status,
        DiagnosticVerificationStatus::Invalid
    );
    assert!(matches!(
        generate_run_diagnosis(&run, &run.join("diagnosis")),
        Err(DiagnosticGenerationError::UnsafeOutput)
    ));
    Ok(())
}

#[cfg(unix)]
#[test]
fn generation_rejects_a_symbolic_link_output_parent() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir()?;
    let run = root.path().join("run");
    let real_output = root.path().join("real-output");
    let linked_output = root.path().join("linked-output");
    fs::create_dir(&run)?;
    fs::create_dir(&real_output)?;
    symlink(&real_output, &linked_output)?;
    write_completed_run(&run)?;
    assert!(matches!(
        generate_run_diagnosis(&run, &linked_output.join("diagnosis")),
        Err(DiagnosticGenerationError::UnsafeOutput)
    ));
    Ok(())
}

fn write_completed_run(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let messages: Vec<ProtocolEnvelope> = serde_json::from_str(include_str!(
        "../../../examples/contracts/protocol-transcript.json"
    ))?;
    let mut recorder = TrajectoryRecorder::new();
    let mut submission: Option<FinalSubmission> = None;
    for message in messages {
        if let ProtocolPayload::FinalSubmission {
            submission: value, ..
        } = &message.payload
        {
            submission = Some(value.clone());
        }
        recorder.append(message)?;
    }
    let submission = submission.ok_or_else(|| io::Error::other("fixture has no submission"))?;
    let trajectory = recorder.as_bytes();
    let mut submission_bytes = serde_json::to_vec_pretty(&submission)?;
    submission_bytes.push(b'\n');
    let metrics = b"{}\n";
    fs::write(root.join("trajectory.jsonl"), trajectory)?;
    fs::write(root.join("submission.json"), &submission_bytes)?;
    fs::write(root.join("metrics.json"), metrics)?;
    let manifest = RunManifest {
        schema_version: SchemaVersion::new(0, 4),
        run_id: RunId::new("run-001")?,
        hashes: BTreeMap::from([
            ("trajectory".into(), Sha256Digest::from_bytes(trajectory)),
            (
                "submission".into(),
                Sha256Digest::from_bytes(&submission_bytes),
            ),
            ("metrics".into(), Sha256Digest::from_bytes(metrics)),
        ]),
        partial: false,
    };
    fs::write(root.join("manifest.json"), serde_json::to_vec(&manifest)?)?;
    Ok(())
}
