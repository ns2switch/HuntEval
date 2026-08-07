mod conclusion;
pub(crate) mod coordination;
mod efficiency;
mod evidence;
mod fingerprint;
mod path;
mod techniques;
mod timeline;

use std::collections::BTreeMap;

use hunteval_domain::MetricValue;

use crate::{EvaluationError, EvaluationInput};

pub(super) fn evaluate_investigation(
    input: &EvaluationInput,
    metrics: &mut BTreeMap<String, MetricValue>,
) -> Result<(), EvaluationError> {
    let (path_precision, path_recall) = path::evaluate(
        &input.submitted_attack_path,
        &input.expected_attack_path,
        input.benign_scored_episode,
    )?;
    metrics.insert("attack_path_precision".into(), path_precision);
    metrics.insert("attack_path_recall".into(), path_recall);

    let (timeline_precision, timeline_recall) = timeline::evaluate(
        input.submitted_timeline.as_deref(),
        input.expected_timeline_windows.as_deref(),
        input.benign_scored_episode,
    )?;
    metrics.insert("timeline_precision".into(), timeline_precision);
    metrics.insert("timeline_recall".into(), timeline_recall);
    metrics.insert(
        "conclusion_correctness".into(),
        conclusion::evaluate(
            input.submitted_status,
            input.acceptable_submission_statuses.as_ref(),
        )?,
    );
    let (technique_precision, technique_recall) = techniques::evaluate(
        &input.submitted_attack_techniques,
        &input.expected_attack_techniques,
        input.benign_scored_episode,
    )?;
    metrics.insert("technique_precision".into(), technique_precision);
    metrics.insert("technique_recall".into(), technique_recall);
    evidence::evaluate(input, metrics);
    coordination::evaluate(input, metrics);
    efficiency::evaluate(input, metrics)?;
    Ok(())
}
