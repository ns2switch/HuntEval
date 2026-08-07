use std::collections::BTreeSet;

use hunteval_domain::{Applicability, MetricDirection, MetricValue, SubmissionStatus};

use crate::{EvaluationError, sets};

pub(super) fn evaluate(
    submitted: SubmissionStatus,
    acceptable: Option<&BTreeSet<SubmissionStatus>>,
) -> Result<MetricValue, EvaluationError> {
    let Some(acceptable) = acceptable else {
        return Ok(sets::unavailable(
            Applicability::AcceptableStatusesUnavailable,
            MetricDirection::HigherIsBetter,
        ));
    };
    if acceptable.is_empty() {
        return Err(EvaluationError::EmptyAcceptableStatuses);
    }
    Ok(sets::ratio(
        u64::from(acceptable.contains(&submitted)),
        1,
        MetricDirection::HigherIsBetter,
    ))
}
