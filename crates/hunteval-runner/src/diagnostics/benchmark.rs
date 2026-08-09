use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use hunteval_domain::{DiagnosticRecurrenceGroup, RunDiagnosis, SchemaVersion, Sha256Digest};
use hunteval_evaluation::{ComparableDiagnosticCell, canonical_taxonomy, reduce_recurrence};
use hunteval_reporting::{
    DiagnosticArtifactKind, DiagnosticArtifactReference, DiagnosticClaim, DiagnosticClaimStage,
    DiagnosticJsonRenderer, DiagnosticReport, DiagnosticReportScope, DiagnosticStaticHtmlRenderer,
    DiagnosticValidationStatus,
};

use super::{
    DiagnosticBundleArtifact, DiagnosticBundleManifest, DiagnosticGenerationError,
    generate_run_diagnosis, service::validate_materialized_destination,
};
use crate::{BenchmarkCellStatus, BenchmarkState, load_stored_definition};

const MAX_STATE_BYTES: u64 = 64 * 1024 * 1024;

pub fn generate_benchmark_diagnosis(
    benchmark_root: &Path,
    output: &Path,
) -> Result<PathBuf, DiagnosticGenerationError> {
    if output.as_os_str().is_empty() || output.starts_with(benchmark_root) || output.exists() {
        return Err(DiagnosticGenerationError::UnsafeOutput);
    }
    let definition = load_stored_definition(benchmark_root)
        .map_err(|_| DiagnosticGenerationError::MalformedSource)?;
    let definition_bytes = read_regular(
        &benchmark_root.join("benchmark-definition.json"),
        MAX_STATE_BYTES,
    )?;
    let state_bytes = read_regular(
        &benchmark_root.join("benchmark-state.json"),
        MAX_STATE_BYTES,
    )?;
    let state: BenchmarkState = serde_json::from_slice(&state_bytes)
        .map_err(|_| DiagnosticGenerationError::MalformedSource)?;
    if state.benchmark_id != definition.id {
        return Err(DiagnosticGenerationError::MalformedSource);
    }
    let parent = output
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(DiagnosticGenerationError::Io)?;
    validate_materialized_destination(benchmark_root, parent)?;
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(DiagnosticGenerationError::UnsafeOutput)?;
    let partial = parent.join(format!(".{name}.partial"));
    if partial.exists() {
        return Err(DiagnosticGenerationError::AlreadyExists);
    }
    fs::create_dir(&partial).map_err(DiagnosticGenerationError::Io)?;
    fs::create_dir_all(partial.join("sources/cells")).map_err(DiagnosticGenerationError::Io)?;
    write_new(
        &partial.join("sources/benchmark-definition.json"),
        &definition_bytes,
    )?;
    write_new(&partial.join("sources/benchmark-state.json"), &state_bytes)?;
    let mut source_files = vec![
        (
            PathBuf::from("sources/benchmark-definition.json"),
            definition_bytes.clone(),
        ),
        (
            PathBuf::from("sources/benchmark-state.json"),
            state_bytes.clone(),
        ),
    ];
    let taxonomy = canonical_taxonomy().map_err(|_| DiagnosticGenerationError::Classification)?;
    let taxonomy_bytes =
        serde_json::to_vec(&taxonomy).map_err(DiagnosticGenerationError::Serialize)?;
    write_new(&partial.join("diagnostic-taxonomy.json"), &taxonomy_bytes)?;
    source_files.push((PathBuf::from("diagnostic-taxonomy.json"), taxonomy_bytes));
    let cells_by_id: BTreeMap<_, _> = definition
        .cells()
        .map_err(|_| DiagnosticGenerationError::MalformedSource)?
        .into_iter()
        .map(|cell| (cell.cell_id, cell))
        .collect();
    let mut comparable = Vec::new();
    let mut generated = Vec::new();
    for cell_state in &state.cells {
        let cell = cells_by_id
            .get(&cell_state.cell_id)
            .ok_or(DiagnosticGenerationError::MalformedSource)?;
        let deployment = Sha256Digest::from_bytes(cell.key.deployment.id.as_str());
        let configuration = cell.key.deployment.configuration_sha256;
        let result_digest = cell_state
            .result_sha256
            .unwrap_or_else(|| Sha256Digest::from_bytes(cell_state.cell_id.to_string().as_bytes()));
        if cell_state.status == BenchmarkCellStatus::Completed {
            let run_id = cell_state
                .run_id
                .as_ref()
                .ok_or(DiagnosticGenerationError::MalformedSource)?;
            let source_run = benchmark_root.join("runs").join(run_id.as_str());
            let result_bytes = read_regular(&source_run.join("result.json"), MAX_STATE_BYTES)?;
            if Sha256Digest::from_bytes(&result_bytes) != result_digest {
                return Err(DiagnosticGenerationError::MalformedSource);
            }
            let result_path =
                PathBuf::from("sources/cells").join(format!("{}.json", cell_state.cell_id));
            write_new(&partial.join(&result_path), &result_bytes)?;
            source_files.push((result_path, result_bytes));
            let relative = PathBuf::from("runs").join(run_id.as_str());
            let target = partial.join(&relative);
            generate_run_diagnosis(&source_run, &target)?;
            let diagnosis_bytes =
                read_regular(&target.join("run-diagnosis.json"), 64 * 1024 * 1024)?;
            let diagnosis: RunDiagnosis = serde_json::from_slice(&diagnosis_bytes)
                .map_err(|_| DiagnosticGenerationError::MalformedSource)?;
            generated.push((
                relative.join("run-diagnosis.json"),
                DiagnosticArtifactKind::RunDiagnosis,
                diagnosis_bytes,
            ));
            comparable.push(ComparableDiagnosticCell {
                cell_id: cell_state.cell_id.to_string(),
                run_id: Some(run_id.clone()),
                deployment_sha256: deployment,
                configuration_sha256: configuration,
                topology_sha256: configuration,
                cell_artifact_sha256: result_digest,
                classifications: diagnosis.classifications,
                exclusion_reason: None,
            });
        } else {
            comparable.push(ComparableDiagnosticCell {
                cell_id: cell_state.cell_id.to_string(),
                run_id: cell_state.run_id.clone(),
                deployment_sha256: deployment,
                configuration_sha256: configuration,
                topology_sha256: configuration,
                cell_artifact_sha256: result_digest,
                classifications: Vec::new(),
                exclusion_reason: Some(
                    cell_state
                        .reason_code
                        .clone()
                        .unwrap_or_else(|| status_reason(cell_state.status).into()),
                ),
            });
        }
    }
    let recurrence =
        reduce_recurrence(&comparable).map_err(|_| DiagnosticGenerationError::Classification)?;
    let recurrence_bytes = json_bytes(&recurrence)?;
    write_new(
        &partial.join("diagnostic-recurrence.json"),
        &recurrence_bytes,
    )?;
    generated.push((
        PathBuf::from("diagnostic-recurrence.json"),
        DiagnosticArtifactKind::Recurrence,
        recurrence_bytes,
    ));
    let report = benchmark_report(
        &definition_bytes,
        &definition.id.to_string(),
        definition.scoring_profile.sha256,
        &recurrence,
        &generated,
    );
    let report_json = DiagnosticJsonRenderer.render(&report)?;
    let report_html = DiagnosticStaticHtmlRenderer.render(&report)?;
    write_new(
        &partial.join("benchmark-diagnostic-report.json"),
        &report_json,
    )?;
    write_new(
        &partial.join("benchmark-diagnostic-report.html"),
        &report_html,
    )?;
    let manifest = benchmark_manifest(
        &partial,
        &generated,
        &source_files,
        &report_json,
        &report_html,
    )?;
    write_new(
        &partial.join("diagnostic-bundle-manifest.json"),
        &json_bytes(&manifest)?,
    )?;
    fs::rename(&partial, output).map_err(DiagnosticGenerationError::Io)?;
    Ok(output.to_path_buf())
}

fn benchmark_report(
    definition_bytes: &[u8],
    benchmark_id: &str,
    scoring_profile: Sha256Digest,
    recurrence: &[DiagnosticRecurrenceGroup],
    generated: &[(PathBuf, DiagnosticArtifactKind, Vec<u8>)],
) -> DiagnosticReport {
    let claims = recurrence
        .iter()
        .map(|group| DiagnosticClaim {
            id: group.id.clone(),
            stage: DiagnosticClaimStage::Observation,
            code: group.code.clone(),
            summary: format!(
                "The observable classification recurred in {} of {} eligible cells.",
                group.occurrences, group.eligible_samples
            ),
            sources: group.sources.clone(),
            validation_status: DiagnosticValidationStatus::NotApplicable,
        })
        .collect();
    let artifacts = generated
        .iter()
        .map(|(path, kind, bytes)| DiagnosticArtifactReference {
            kind: *kind,
            path: path.to_string_lossy().replace('\\', "/"),
            sha256: Sha256Digest::from_bytes(bytes),
        })
        .collect();
    DiagnosticReport {
        schema_version: SchemaVersion::new(0, 7),
        report_id: format!(
            "diagnostic-report:{}",
            Sha256Digest::from_bytes(definition_bytes)
        ),
        scope: DiagnosticReportScope::Benchmark,
        subject_id: benchmark_id.into(),
        source_manifest_sha256: Sha256Digest::from_bytes(definition_bytes),
        metric_vector_sha256: None,
        scoring_profile_sha256: Some(scoring_profile),
        claims,
        artifacts,
        limitations: [
            "recurrence_is_not_causality".into(),
            "topology_identity_is_bound_by_deployment_configuration".into(),
        ]
        .into_iter()
        .collect(),
    }
}

fn benchmark_manifest(
    partial: &Path,
    generated: &[(PathBuf, DiagnosticArtifactKind, Vec<u8>)],
    source_files: &[(PathBuf, Vec<u8>)],
    report_json: &[u8],
    report_html: &[u8],
) -> Result<DiagnosticBundleManifest, DiagnosticGenerationError> {
    let mut artifacts = Vec::new();
    for (path, _, bytes) in generated {
        artifacts.push(bundle_artifact(path, "application/json", bytes));
    }
    for (path, bytes) in source_files {
        artifacts.push(bundle_artifact(path, "application/json", bytes));
    }
    let runs = partial.join("runs");
    if runs.exists() {
        for entry in fs::read_dir(&runs).map_err(DiagnosticGenerationError::Io)? {
            let entry = entry.map_err(DiagnosticGenerationError::Io)?;
            let run_root = entry.path();
            let run_name = entry.file_name();
            for file in fs::read_dir(&run_root).map_err(DiagnosticGenerationError::Io)? {
                let file = file.map_err(DiagnosticGenerationError::Io)?;
                let path = file.path();
                if !path.is_file() || path.ends_with("run-diagnosis.json") {
                    continue;
                }
                let bytes = read_regular(&path, 64 * 1024 * 1024)?;
                let relative = PathBuf::from("runs").join(&run_name).join(file.file_name());
                let media = match path.extension().and_then(|value| value.to_str()) {
                    Some("html") => "text/html",
                    Some("jsonl") => "application/x-ndjson",
                    _ => "application/json",
                };
                artifacts.push(bundle_artifact(&relative, media, &bytes));
            }
        }
    }
    artifacts.push(bundle_artifact(
        Path::new("benchmark-diagnostic-report.json"),
        "application/json",
        report_json,
    ));
    artifacts.push(bundle_artifact(
        Path::new("benchmark-diagnostic-report.html"),
        "text/html",
        report_html,
    ));
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    let taxonomy = canonical_taxonomy().map_err(|_| DiagnosticGenerationError::Classification)?;
    Ok(DiagnosticBundleManifest {
        schema_version: SchemaVersion::new(0, 7),
        bundle_id: format!(
            "diagnostic-bundle:{}",
            Sha256Digest::from_bytes(
                serde_json::to_vec(&artifacts).map_err(DiagnosticGenerationError::Serialize)?
            )
        ),
        source_revision: Sha256Digest::from_bytes(env!("CARGO_PKG_VERSION")),
        taxonomy_sha256: taxonomy
            .digest()
            .map_err(|_| DiagnosticGenerationError::Classification)?,
        classifier_registry_sha256: hunteval_evaluation::classifier_registry_digest(),
        artifacts,
        root_reports: [
            "benchmark-diagnostic-report.json".into(),
            "benchmark-diagnostic-report.html".into(),
        ]
        .into_iter()
        .collect(),
        limitations: ["recurrence_is_not_causality".into()].into_iter().collect(),
    })
}

fn bundle_artifact(path: &Path, media: &str, bytes: &[u8]) -> DiagnosticBundleArtifact {
    DiagnosticBundleArtifact {
        path: path.to_string_lossy().replace('\\', "/"),
        media_type: media.into(),
        size_bytes: bytes.len() as u64,
        sha256: Sha256Digest::from_bytes(bytes),
    }
}

const fn status_reason(status: BenchmarkCellStatus) -> &'static str {
    match status {
        BenchmarkCellStatus::Pending => "cell_pending",
        BenchmarkCellStatus::Running => "cell_running",
        BenchmarkCellStatus::Completed => "cell_completed_without_result",
        BenchmarkCellStatus::Failed => "cell_failed",
        BenchmarkCellStatus::NonComparable => "cell_non_comparable",
    }
}

fn read_regular(path: &Path, maximum: u64) -> Result<Vec<u8>, DiagnosticGenerationError> {
    let metadata = fs::symlink_metadata(path).map_err(DiagnosticGenerationError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum {
        return Err(DiagnosticGenerationError::MalformedSource);
    }
    fs::read(path).map_err(DiagnosticGenerationError::Io)
}

fn json_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, DiagnosticGenerationError> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(DiagnosticGenerationError::Serialize)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), DiagnosticGenerationError> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(DiagnosticGenerationError::Io)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(DiagnosticGenerationError::Io)
}
