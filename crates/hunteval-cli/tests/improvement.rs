use std::process::Command;

#[test]
fn improvement_help_exposes_controlled_workflow_without_adoption_command()
-> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_hunteval"))
        .args(["improvement", "--help"])
        .output()?;
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout)?;
    for command in ["validate", "run", "resume", "status", "verify"] {
        assert!(help.contains(command), "missing command {command}");
    }
    assert!(!help.contains("adopt"));
    Ok(())
}
