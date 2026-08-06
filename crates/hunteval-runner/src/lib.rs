//! Trusted HuntEval orchestration components.
//!
//! PR-04 exposes only the validated episode-package loader. Process execution,
//! budgets, and artifact recording are introduced in later milestones.

mod episode_loader;

pub use episode_loader::{ArtifactDigests, EpisodeLoadError, EpisodePackage, PublicEpisodePackage};
