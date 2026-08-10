use std::path::Path;

use hunteval_domain::{ArtifactKind, ArtifactMediaType, ArtifactProvenance};
use hunteval_runner::{
    ArtifactRegistrationRequest, ArtifactRegistryError, register_artifact,
    verify_registered_artifact, write_artifact_registry,
};

#[test]
fn registers_deduplicates_and_verifies_exact_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let public = temp.path().join("public");
    let registry = temp.path().join("registry");
    std::fs::create_dir(&public)?;
    std::fs::write(
        public.join("instruction.json"),
        b"{\"role\":\"supervisor\"}",
    )?;
    let request = ArtifactRegistrationRequest {
        id: "supervisor-instruction",
        kind: ArtifactKind::Instruction,
        media_type: ArtifactMediaType::Json,
        label: "supervisor instruction",
        provenance: ArtifactProvenance::Repository,
        structured_artifact_sha256: None,
    };
    let first = register_artifact(&public, Path::new("instruction.json"), &registry, &request)?;
    let second = register_artifact(&public, Path::new("instruction.json"), &registry, &request)?;
    assert_eq!(first, second);
    let registry_digest = write_artifact_registry(&registry, "registry", vec![first.clone()])?;
    assert_eq!(
        registry_digest,
        hunteval_domain::Sha256Digest::from_bytes(std::fs::read(
            registry.join("artifact-registry.json")
        )?)
    );
    verify_registered_artifact(&registry, &first)?;
    std::fs::write(
        registry.join("artifacts").join(first.sha256.to_string()),
        b"tampered",
    )?;
    assert_eq!(
        verify_registered_artifact(&registry, &first),
        Err(ArtifactRegistryError::DigestMismatch)
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn rejects_traversal_symlinks_and_hardlinks() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir()?;
    let public = temp.path().join("public");
    let registry = temp.path().join("registry");
    std::fs::create_dir(&public)?;
    std::fs::write(public.join("source.txt"), "safe")?;
    std::fs::hard_link(public.join("source.txt"), public.join("hard.txt"))?;
    symlink(public.join("source.txt"), public.join("link.txt"))?;
    let request = ArtifactRegistrationRequest {
        id: "artifact",
        kind: ArtifactKind::Instruction,
        media_type: ArtifactMediaType::Text,
        label: "artifact",
        provenance: ArtifactProvenance::Repository,
        structured_artifact_sha256: None,
    };
    for path in ["../source.txt", "link.txt", "hard.txt"] {
        assert_eq!(
            register_artifact(&public, Path::new(path), &registry, &request),
            Err(ArtifactRegistryError::UnsafeSource)
        );
    }
    Ok(())
}
