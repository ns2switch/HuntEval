"""Finite read-only operation catalogs for commercial connector previews."""

from __future__ import annotations

READ_ONLY_OPERATIONS: dict[str, frozenset[str]] = {
    "crowdstrike_falcon": frozenset(
        {"detections_search", "detections_get", "incidents_get", "threat_intel_search"}
    ),
    "google_secops": frozenset(
        {"udm_query_validate", "udm_search", "events_get", "alerts_get", "cases_get"}
    ),
    "microsoft_sentinel": frozenset(
        {"hunting_query", "incidents_get", "alerts_get", "entities_get"}
    ),
    "elastic_security": frozenset(
        {"security_search", "alerts_get", "investigations_get"}
    ),
    "cortex_xsiam": frozenset(
        {"alerts_get", "incidents_get", "queries_run", "audit_get"}
    ),
}


def operations_for(platform: str) -> frozenset[str]:
    """Return the exact immutable read-only catalog for one platform."""
    operations = READ_ONLY_OPERATIONS.get(platform)
    if operations is None:
        raise ValueError("commercial platform is unsupported")
    return operations
