use serde::{Deserialize, Serialize};

/// Commercial platforms with an explicit preview catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommercialPlatform {
    CrowdstrikeFalcon,
    GoogleSecops,
    MicrosoftSentinel,
    ElasticSecurity,
    CortexXsiam,
}

/// Finite read-only operations. Mutation operations are not representable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommercialOperation {
    DetectionsSearch,
    DetectionsGet,
    IncidentsGet,
    ThreatIntelSearch,
    UdmQueryValidate,
    UdmSearch,
    EventsGet,
    AlertsGet,
    CasesGet,
    HuntingQuery,
    EntitiesGet,
    SecuritySearch,
    InvestigationsGet,
    QueriesRun,
    AuditGet,
}

impl CommercialPlatform {
    /// Whether the operation belongs to the exact platform catalog.
    #[must_use]
    pub const fn supports(self, operation: CommercialOperation) -> bool {
        use CommercialOperation as O;
        match self {
            Self::CrowdstrikeFalcon => matches!(
                operation,
                O::DetectionsSearch | O::DetectionsGet | O::IncidentsGet | O::ThreatIntelSearch
            ),
            Self::GoogleSecops => matches!(
                operation,
                O::UdmQueryValidate | O::UdmSearch | O::EventsGet | O::AlertsGet | O::CasesGet
            ),
            Self::MicrosoftSentinel => matches!(
                operation,
                O::HuntingQuery | O::IncidentsGet | O::AlertsGet | O::EntitiesGet
            ),
            Self::ElasticSecurity => matches!(
                operation,
                O::SecuritySearch | O::AlertsGet | O::InvestigationsGet
            ),
            Self::CortexXsiam => matches!(
                operation,
                O::AlertsGet | O::IncidentsGet | O::QueriesRun | O::AuditGet
            ),
        }
    }
}
