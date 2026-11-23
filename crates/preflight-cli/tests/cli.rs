use std::process::Command;
fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_preflight"))
}
#[test]
fn help_works() {
    let out = cli().arg("--help").output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("check"))
}
#[test]
fn missing_path_produces_exit_one_and_safe_json() {
    let output = cli()
        .args([
            "check",
            "--profile",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../profiles/python-workshop.yaml"
            ),
            "--format",
            "json",
        ])
        .env_remove("PATH")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let j: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(j["schema_version"], 1);
    assert_eq!(j["results"][0]["reason_code"], "not_found");
    assert_eq!(j["summary"]["ready"], false);
    assert!(output.stderr.is_empty())
}
#[test]
fn unreadable_profile_returns_safe_error() {
    let output = cli()
        .args([
            "check",
            "--profile",
            "/secret-user/missing-profile.yaml",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let s = String::from_utf8(output.stdout).unwrap();
    assert!(!s.contains("secret-user"));
    let j: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(j["reason_code"], "profile_unreadable")
}
#[test]
fn invalid_profile_exit_two() {
    let output = cli()
        .args([
            "check",
            "--profile",
            concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let j: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(j["reason_code"], "invalid_profile")
}
#[test]
fn text_is_slovenian() {
    let output = cli()
        .args([
            "check",
            "--profile",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../profiles/python-workshop.yaml"
            ),
            "--format",
            "text",
        ])
        .env_remove("PATH")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Okolje še ni pripravljeno"))
}
