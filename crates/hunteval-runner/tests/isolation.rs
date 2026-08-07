use std::{collections::BTreeMap, fs};

use std::time::Duration;

use hunteval_runner::{IsolationPolicy, LinuxSandbox, PolicyError};

#[test]
fn permits_only_allowlisted_environment_names() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let mut environment = BTreeMap::new();
    environment.insert("HUNTEVAL_RUN_ID".into(), "run-1".into());
    assert!(IsolationPolicy::new(root.path().to_path_buf(), environment).is_ok());
    let mut invalid = BTreeMap::new();
    invalid.insert("LD_PRELOAD".to_lowercase(), "bad".into());
    assert!(matches!(
        IsolationPolicy::new(root.path().to_path_buf(), invalid),
        Err(PolicyError::EnvironmentKey(_))
    ));
    Ok(())
}

#[test]
fn linux_backend_hides_private_root_and_network() -> Result<(), Box<dyn std::error::Error>> {
    if !std::path::Path::new("/usr/bin/bwrap").is_file() {
        return Ok(());
    }
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("public.txt"), b"visible")?;
    let policy = IsolationPolicy::new(root.path().to_path_buf(), BTreeMap::new())?;
    let arguments = vec![
        "-c".into(),
        "test -f /episode/public.txt && test ! -e /root/hunteval/AGENTS.md && test ! -e /sys/class/net/eth0".into(),
    ];
    let output = LinuxSandbox::run(
        std::path::Path::new("/bin/sh"),
        &arguments,
        &policy,
        Duration::from_secs(2),
        1024,
    )?;
    assert_eq!(output.exit_code, 0);
    Ok(())
}

#[test]
fn rejects_traversal_and_symlink_escape() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("public.json"), b"{}")?;
    let policy = IsolationPolicy::new(root.path().to_path_buf(), BTreeMap::new())?;
    assert!(
        policy
            .resolve_public(std::path::Path::new("public.json"))
            .is_ok()
    );
    assert!(matches!(
        policy.resolve_public(std::path::Path::new("../private.json")),
        Err(PolicyError::PathTraversal)
    ));
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/etc/passwd", root.path().join("escape"))?;
        assert!(matches!(
            policy.resolve_public(std::path::Path::new("escape")),
            Err(PolicyError::PathTraversal)
        ));
    }
    Ok(())
}
