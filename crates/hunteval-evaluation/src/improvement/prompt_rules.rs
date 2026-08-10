use hunteval_domain::{
    DiffOperationKind as Operation, MutableSectionClass as Target,
    ObservableSourceFamily as Source, PromptWeaknessCode as Weakness,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct CompiledPromptRule {
    pub weakness: Weakness,
    pub diagnostic: &'static str,
    pub sources: &'static [Source],
    pub targets: &'static [Target],
    pub operations: &'static [Operation],
}

pub(super) const RULES: &[CompiledPromptRule] = &[
    rule(
        Weakness::RoleAmbiguity,
        "capability_mismatch",
        &[Source::Agent, Source::Task],
        &[Target::TaskPlanning],
        &[Operation::AddConstraint, Operation::ReplaceSection],
    ),
    rule(
        Weakness::MissingOutputContract,
        "invalid_output",
        &[Source::Trajectory],
        &[Target::OutputContract],
        &[Operation::AddSection],
    ),
    rule(
        Weakness::MissingEvidenceRequirements,
        "ungrounded_finding",
        &[Source::Finding, Source::Evidence],
        &[Target::EvidenceRequirements],
        &[Operation::AddConstraint],
    ),
    rule(
        Weakness::MissingAcceptanceCriteria,
        "unsupported_conclusion",
        &[Source::Finding, Source::Metric],
        &[Target::EvidenceRequirements, Target::OutputContract],
        &[Operation::AddConstraint],
    ),
    rule(
        Weakness::MissingStoppingCondition,
        "tool_budget_exhausted",
        &[Source::Action, Source::Metric],
        &[Target::StoppingConditions],
        &[Operation::AddSection, Operation::AddConstraint],
    ),
    rule(
        Weakness::UnclearToolUsePolicy,
        "repeated_query",
        &[Source::Action],
        &[Target::TaskPlanning],
        &[Operation::AddConstraint],
    ),
    rule(
        Weakness::InsufficientErrorHandling,
        "ignored_tool_error",
        &[Source::Action, Source::Trajectory],
        &[Target::ErrorRecovery],
        &[Operation::AddSection, Operation::ReplaceSection],
    ),
    rule(
        Weakness::InsufficientDelegationPolicy,
        "incorrect_delegation",
        &[Source::Agent, Source::Task],
        &[Target::DelegationStrategy],
        &[Operation::AddConstraint],
    ),
    rule(
        Weakness::DuplicatedResponsibility,
        "duplicate_task_creation",
        &[Source::Agent, Source::Task],
        &[Target::DelegationStrategy],
        &[Operation::ReplaceSection],
    ),
    rule(
        Weakness::MissingTaskOwnership,
        "duplicate_task_creation",
        &[Source::Task, Source::Coordination],
        &[Target::DelegationStrategy],
        &[Operation::AddConstraint],
    ),
    rule(
        Weakness::MissingConflictResolutionPolicy,
        "unresolved_conflict",
        &[Source::Coordination, Source::Finding],
        &[Target::CommunicationFormat],
        &[Operation::AddSection, Operation::AddConstraint],
    ),
    rule(
        Weakness::ExcessiveCommunicationRequirements,
        "excessive_message_loop",
        &[Source::Coordination, Source::Metric],
        &[Target::CommunicationFormat],
        &[Operation::ReplaceSection],
    ),
    rule(
        Weakness::InsufficientEvidenceSharingRules,
        "evidence_not_shared",
        &[Source::Evidence, Source::Coordination],
        &[Target::EvidenceRequirements, Target::CommunicationFormat],
        &[Operation::AddConstraint],
    ),
    rule(
        Weakness::OverlyBroadSpecialistInvocationCriteria,
        "unnecessary_specialist_invocation",
        &[Source::Agent, Source::Task, Source::Metric],
        &[Target::DelegationStrategy],
        &[Operation::ReplaceSection, Operation::AddConstraint],
    ),
];

const fn rule(
    weakness: Weakness,
    diagnostic: &'static str,
    sources: &'static [Source],
    targets: &'static [Target],
    operations: &'static [Operation],
) -> CompiledPromptRule {
    CompiledPromptRule {
        weakness,
        diagnostic,
        sources,
        targets,
        operations,
    }
}
