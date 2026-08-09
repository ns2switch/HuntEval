use std::{fs, path::Path, process::ExitCode};

use crate::args::ReportFormatArgument;

#[derive(Debug, Clone, Copy)]
pub(super) struct InputPaths<'a> {
    pub experiment: &'a Path,
    pub baseline_topology: &'a Path,
    pub candidate_topology: &'a Path,
    pub statistical_policy: &'a Path,
    pub scoring_profile: &'a Path,
    pub observations: &'a Path,
}

pub(super) fn render(
    paths: InputPaths<'_>,
    seed: u64,
    format: ReportFormatArgument,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let experiment = read_bounded(paths.experiment)?;
    let baseline_topology = read_bounded(paths.baseline_topology)?;
    let candidate_topology = read_bounded(paths.candidate_topology)?;
    let statistical_policy = read_bounded(paths.statistical_policy)?;
    let scoring_profile = read_bounded(paths.scoring_profile)?;
    let observations = read_bounded(paths.observations)?;
    let rendered = hunteval_runner::render_controlled_topology_report(
        hunteval_runner::ControlledTopologyReportInput {
            experiment: &experiment,
            baseline_topology: &baseline_topology,
            candidate_topology: &candidate_topology,
            statistical_policy: &statistical_policy,
            scoring_profile: &scoring_profile,
            observations: &observations,
            seed,
            format: match format {
                ReportFormatArgument::Json => hunteval_runner::ReportFormat::Json,
                ReportFormatArgument::Html => hunteval_runner::ReportFormat::Html,
            },
        },
    )?;
    print!("{}", String::from_utf8(rendered)?);
    Ok(ExitCode::SUCCESS)
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 64 * 1024 * 1024
    {
        return Err(
            std::io::Error::other("topology report input is not a bounded regular file").into(),
        );
    }
    Ok(fs::read(path)?)
}
