use std::collections::BTreeSet;

use hunteval_domain::{MetricDirection, MetricValue, ResourceProvenance, SchemaVersion};

use crate::{
    MetricSelection, ResourceProvenanceRequirement, ScoringConstraint, ScoringProfile,
    ScoringProfileArtifact, metric_contract,
};

use super::ProfileError;

const CURRENT_PROFILE_VERSION: SchemaVersion = SchemaVersion::new(0, 4);
const LEGACY_PROFILE_VERSION: SchemaVersion = SchemaVersion::new(0, 3);

pub fn normalize_profile(artifact: ScoringProfileArtifact) -> Result<ScoringProfile, ProfileError> {
    let profile = match artifact {
        ScoringProfileArtifact::Current(profile) => profile,
        ScoringProfileArtifact::Legacy(legacy) => {
            if legacy.schema_version != LEGACY_PROFILE_VERSION {
                return Err(ProfileError::UnsupportedProfileVersion(
                    legacy.schema_version,
                ));
            }
            ScoringProfile {
                schema_version: CURRENT_PROFILE_VERSION,
                id: legacy.id,
                missing_metric_policy: legacy.missing_metric_policy,
                metrics: legacy
                    .weights
                    .into_iter()
                    .map(|(name, weight)| {
                        (
                            name,
                            MetricSelection {
                                version: LEGACY_PROFILE_VERSION,
                                weight,
                            },
                        )
                    })
                    .collect(),
                constraints: legacy
                    .disqualifying_constraints
                    .into_iter()
                    .map(|code| ScoringConstraint::ObservedViolation {
                        code,
                        disqualifying: true,
                    })
                    .collect(),
            }
        }
    };
    validate_profile(&profile)?;
    Ok(profile)
}

pub(super) fn validate_profile(profile: &ScoringProfile) -> Result<(), ProfileError> {
    if profile.schema_version != CURRENT_PROFILE_VERSION {
        return Err(ProfileError::UnsupportedProfileVersion(
            profile.schema_version,
        ));
    }
    if profile.id.is_empty()
        || profile.id.len() > 128
        || !profile
            .id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
        || profile.metrics.is_empty()
        || profile.metrics.len() > 64
        || profile.constraints.len() > 128
    {
        return Err(ProfileError::InvalidProfile);
    }
    for (name, selection) in &profile.metrics {
        if metric_contract(name, selection.version).is_none() {
            return Err(ProfileError::UnknownMetricVersion(
                name.clone(),
                selection.version,
            ));
        }
    }
    let total = profile
        .metrics
        .values()
        .map(|selection| selection.weight)
        .sum::<f64>();
    if profile
        .metrics
        .values()
        .any(|selection| !selection.weight.is_finite() || selection.weight < 0.0)
        || (total - 1.0).abs() > 1e-9
    {
        return Err(ProfileError::InvalidWeightsOrMetric);
    }
    let mut codes = BTreeSet::new();
    for constraint in &profile.constraints {
        validate_constraint(constraint, &mut codes)?;
    }
    Ok(())
}

pub(super) fn validate_metric_value(
    name: &str,
    metric: &MetricValue,
    direction: MetricDirection,
) -> Result<(), ProfileError> {
    metric
        .validate()
        .map_err(|_| ProfileError::MetricContractMismatch(name.to_owned()))?;
    if metric.direction != direction || metric.range.minimum != 0.0 || metric.range.maximum != 1.0 {
        return Err(ProfileError::MetricContractMismatch(name.to_owned()));
    }
    Ok(())
}

fn validate_constraint(
    constraint: &ScoringConstraint,
    codes: &mut BTreeSet<String>,
) -> Result<(), ProfileError> {
    let code = match constraint {
        ScoringConstraint::ObservedViolation { code, .. } => code,
        ScoringConstraint::MetricThreshold {
            code,
            metric,
            threshold,
            required_resource_provenance,
            ..
        } => {
            if !threshold.is_finite() || !(0.0..=1.0).contains(threshold) {
                return Err(ProfileError::InvalidConstraint);
            }
            let contract = metric_contract(&metric.name, metric.version).ok_or_else(|| {
                ProfileError::UnknownMetricVersion(metric.name.clone(), metric.version)
            })?;
            if requirement_for(contract.required_resource_provenance)
                != *required_resource_provenance
            {
                return Err(ProfileError::InvalidConstraint);
            }
            code
        }
    };
    if code.is_empty()
        || code.len() > 128
        || !code
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        || !codes.insert(code.clone())
    {
        return Err(ProfileError::InvalidConstraint);
    }
    Ok(())
}

const fn requirement_for(provenance: Option<ResourceProvenance>) -> ResourceProvenanceRequirement {
    match provenance {
        None => ResourceProvenanceRequirement::None,
        Some(ResourceProvenance::Measured) => ResourceProvenanceRequirement::Measured,
        Some(ResourceProvenance::VerifiedAdapter) => ResourceProvenanceRequirement::VerifiedAdapter,
        Some(ResourceProvenance::SelfReported | ResourceProvenance::Unavailable) => {
            ResourceProvenanceRequirement::None
        }
    }
}
