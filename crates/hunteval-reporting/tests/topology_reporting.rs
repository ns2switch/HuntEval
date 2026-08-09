use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    SchemaVersion, Sha256Digest, TopologyAnalysis, TopologyAnalysisKind,
    TopologyMetricApplicability, TopologyMetricValue,
};
use hunteval_reporting::{ConstraintFirstStatus, TopologyComparisonReport};
use hunteval_statistics::{
    CalibrationPolicy, ComparisonClass, EffectSizeMethod, IntervalMethod, MultiplicityMethod,
    MultiplicityPolicy, StatisticalPolicy, compare_with_policy,
};

#[test]
fn topology_report_is_deterministic_escaped_and_explicitly_limited()
-> Result<(), Box<dyn std::error::Error>> {
    let policy_sha256 = Sha256Digest::from_bytes(b"statistics");
    let policy = StatisticalPolicy {
        schema_version: SchemaVersion::new(0, 6),
        id: hunteval_domain::StatisticalPolicyId::new("topology-report-policy")?,
        comparison_class: ComparisonClass::Validation,
        minimum_paired_samples: 2,
        confidence_level: 0.95,
        interval_method: IntervalMethod::DeterministicPairedBootstrap,
        effect_size_method: EffectSizeMethod::PairedMeanDifference,
        multiplicity: MultiplicityPolicy {
            method: MultiplicityMethod::HolmBonferroni,
            family: "topology-report".to_owned(),
        },
        calibration: CalibrationPolicy::NotRequired,
    };
    let report = TopologyComparisonReport {
        schema_version: SchemaVersion::new(0, 6),
        analysis: TopologyAnalysis {
            schema_version: SchemaVersion::new(0, 6),
            baseline_topology_sha256: Sha256Digest::from_bytes(b"baseline"),
            candidate_topology_sha256: Sha256Digest::from_bytes(b"candidate"),
            experiment_sha256: Some(Sha256Digest::from_bytes(b"experiment")),
            analysis_kind: TopologyAnalysisKind::ControlledAblation,
            topology_dependent: true,
            metrics: BTreeMap::from([
                (
                    "investigation_quality".to_owned(),
                    TopologyMetricValue {
                        applicability: TopologyMetricApplicability::Applicable,
                        value: Some(0.8),
                        reason_code: None,
                    },
                ),
                (
                    "role_contribution".to_owned(),
                    TopologyMetricValue {
                        applicability: TopologyMetricApplicability::Unavailable,
                        value: None,
                        reason_code: Some("insufficient_controlled_samples".to_owned()),
                    },
                ),
            ]),
            limitations: BTreeSet::from(["experimental_topology_dependent".to_owned()]),
        },
        statistical_policy_sha256: policy_sha256,
        scoring_profile_sha256: Sha256Digest::from_bytes(b"scoring"),
        comparisons: BTreeMap::from([(
            "investigation_quality".to_owned(),
            compare_with_policy(
                &policy,
                policy_sha256,
                &[Some(0.8), Some(0.8)],
                &[Some(0.5), Some(0.5)],
                7,
            )?,
        )]),
        aggregate_score: Some(0.8),
        constraint_first_status: ConstraintFirstStatus::CandidatePreferred,
        limitations: vec!["No universal role transfer; <script>alert(1)</script>".to_owned()],
    };
    assert_eq!(report.render_json()?, report.render_json()?);
    let html = String::from_utf8(report.render_html()?)?;
    assert!(html.contains("experimental and topology-dependent"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>"));
    assert!(html.contains("Raw metrics are authoritative"));
    assert!(html.contains("Paired samples"));
    assert!(html.contains("conclusive"));
    Ok(())
}
