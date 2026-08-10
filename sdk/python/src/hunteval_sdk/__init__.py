"""Typed, offline helpers for public HuntEval contracts."""

from .artifacts import VerifiedPublicArtifact, read_public_artifact, read_verified_json
from .contracts import (
    AnalyticalQuery,
    AnalyticalResult,
    ExtensionCapabilityPolicy,
    ExtensionConformanceResult,
    ExtensionResolution,
    RetrievalAuditEvent,
)
from .crewai import CrewAIAdapter, CrewAIAdapterConfig, CrewAIContext, CrewFactory, CrewLike
from .models import AnalyticalCorpus, CorpusSource, ExtensionManifest
from .protocol import DeploymentPeer, FinalSubmissionMessage, ProtocolError, RegistrationMessage
from .tool_contracts import ManagedToolAdapterRequest, ManagedToolAdapterResponse

__all__ = [
    "AnalyticalCorpus",
    "AnalyticalQuery",
    "AnalyticalResult",
    "CorpusSource",
    "CrewAIAdapter",
    "CrewAIAdapterConfig",
    "CrewAIContext",
    "CrewFactory",
    "CrewLike",
    "DeploymentPeer",
    "ExtensionManifest",
    "ExtensionCapabilityPolicy",
    "ExtensionConformanceResult",
    "ExtensionResolution",
    "FinalSubmissionMessage",
    "ManagedToolAdapterRequest",
    "ManagedToolAdapterResponse",
    "ProtocolError",
    "RegistrationMessage",
    "RetrievalAuditEvent",
    "VerifiedPublicArtifact",
    "read_public_artifact",
    "read_verified_json",
]
