use preflight_core::{engine::Platform, process::*, *};
use std::{collections::VecDeque, sync::Mutex};
struct Fake(Mutex<VecDeque<(Invocation, Result<Output, ProcessError>)>>);
impl Fake {
    fn new(items: Vec<(Invocation, Result<Output, ProcessError>)>) -> Self {
        Self(Mutex::new(items.into()))
    }
    fn done(&self) {
        assert!(self.0.lock().unwrap().is_empty())
    }
}
impl ProcessRunner for Fake {
    async fn run(&self, i: Invocation) -> Result<Output, ProcessError> {
        let (expected, result) = self
            .0
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected command");
        assert_eq!(i, expected);
        result
    }
}
fn out(s: &str, e: &str) -> Result<Output, ProcessError> {
    Ok(Output {
        stdout: s.into(),
        stderr: e.into(),
    })
}
fn profile(tool: &str, required: bool) -> Profile {
    Profile::parse(&format!("schema_version: 1\nid: test\nname: Test\nchecks:\n  - id: runtime\n    type: runtime\n    tool: {tool}\n    version: '>=2.0.0'\n    required: {required}\n")).unwrap()
}
#[tokio::test]
async fn windows_launcher_fallback() {
    let f = Fake::new(vec![
        (Invocation::Python, Err(ProcessError::NotFound)),
        (Invocation::Py3, out("", "Python 3.12.7")),
    ]);
    let r = check_profile(&profile("python", true), &f, Platform::Windows).await;
    assert_eq!(r.exit_code(), 0);
    assert_eq!(r.results[0].installed_version.as_deref(), Some("3.12.7"));
    f.done()
}
#[tokio::test]
async fn unix_prefers_python3() {
    let f = Fake::new(vec![(Invocation::Python3, out("Python 3.11.0", ""))]);
    assert_eq!(
        check_profile(&profile("python", true), &f, Platform::Unix)
            .await
            .exit_code(),
        0
    );
    f.done()
}
#[tokio::test]
async fn no_path_is_missing() {
    let f = Fake::new(vec![
        (Invocation::Python3, Err(ProcessError::NotFound)),
        (Invocation::Python, Err(ProcessError::NotFound)),
    ]);
    let r = check_profile(&profile("python", true), &f, Platform::Unix).await;
    assert_eq!(r.results[0].reason_code, ReasonCode::NotFound);
    assert_eq!(r.exit_code(), 1);
    f.done()
}
#[tokio::test]
async fn java_stderr_and_jdk() {
    for version in [
        "openjdk version \"21.0.2\" 2024-01-16",
        "java version \"21.0.2\"",
        "openjdk 21.0.2 2024-01-16",
    ] {
        let f = Fake::new(vec![
            (Invocation::Java, out("", version)),
            (Invocation::Javac, out("javac 21.0.2", "")),
        ]);
        assert_eq!(
            check_profile(&profile("java", true), &f, Platform::Unix)
                .await
                .exit_code(),
            0
        );
        f.done()
    }
}
#[tokio::test]
async fn java_requires_compiler() {
    let f = Fake::new(vec![
        (Invocation::Java, out("", "java version \"21.0.2\"")),
        (Invocation::Javac, Err(ProcessError::NotFound)),
    ]);
    let r = check_profile(&profile("java", true), &f, Platform::Unix).await;
    assert_eq!(r.results[0].reason_code, ReasonCode::JdkMissing);
    assert_eq!(r.exit_code(), 1)
}
#[tokio::test]
async fn mismatched_jdk() {
    let f = Fake::new(vec![
        (Invocation::Java, out("", "openjdk 21.0.2")),
        (Invocation::Javac, out("javac 17.0.10", "")),
    ]);
    assert_eq!(
        check_profile(&profile("java", true), &f, Platform::Unix)
            .await
            .results[0]
            .reason_code,
        ReasonCode::JdkMismatch
    )
}
#[tokio::test]
async fn failures_are_unknown() {
    for (error, reason) in [
        (ProcessError::Timeout, ReasonCode::Timeout),
        (ProcessError::OutputLimit, ReasonCode::OutputLimit),
        (ProcessError::Failed, ReasonCode::ProcessFailed),
    ] {
        let f = Fake::new(vec![(Invocation::Git, Err(error))]);
        let r = check_profile(&profile("git", true), &f, Platform::Unix).await;
        assert_eq!(r.results[0].status, Status::Unknown);
        assert_eq!(r.results[0].reason_code, reason);
        assert_eq!(r.exit_code(), 1)
    }
}
#[tokio::test]
async fn optional_missing_is_warning() {
    let f = Fake::new(vec![(Invocation::Node, Err(ProcessError::NotFound))]);
    let r = check_profile(&profile("node", false), &f, Platform::Unix).await;
    assert_eq!(r.results[0].status, Status::Warning);
    assert_eq!(r.exit_code(), 0)
}
#[tokio::test]
async fn version_constraint_and_privacy() {
    let yaml = include_str!("fixtures/runtime.yaml");
    let f = Fake::new(vec![
        (
            Invocation::Python3,
            out("Python 3.12.7", "private C:\\Users\\secret-user"),
        ),
        (
            Invocation::Git,
            out("git version 2.53.0.windows.1", "machine-id-secret"),
        ),
        (Invocation::Node, out("v24.15.0", "")),
    ]);
    let r = check_profile(&Profile::parse(yaml).unwrap(), &f, Platform::Unix).await;
    assert_eq!(r.summary.pass, 2);
    assert_eq!(r.summary.warning, 1);
    assert_eq!(r.exit_code(), 0);
    let json = serde_json::to_string(&r).unwrap();
    for forbidden in [
        "secret-user",
        "machine-id",
        "Users",
        "stdout",
        "stderr",
        "private",
    ] {
        assert!(!json.contains(forbidden))
    }
    assert_eq!(serde_json::to_string(&r).unwrap(), json);
    f.done()
}
#[tokio::test]
async fn required_version_mismatch() {
    let f = Fake::new(vec![(Invocation::Git, out("git version 1.9.0", ""))]);
    assert_eq!(
        check_profile(&profile("git", true), &f, Platform::Unix)
            .await
            .results[0]
            .reason_code,
        ReasonCode::VersionMismatch
    )
}
#[tokio::test]
async fn unknown_version_never_passes() {
    let f = Fake::new(vec![(Invocation::Node, out("v22.x", ""))]);
    let r = check_profile(&profile("node", true), &f, Platform::Unix).await;
    assert_eq!(r.results[0].status, Status::Unknown);
    assert_eq!(r.exit_code(), 1)
}
#[test]
fn rejects_invalid_profiles() {
    let valid = include_str!("fixtures/runtime.yaml");
    for yaml in [
        valid.replace("schema_version: 1", "schema_version: 2"),
        valid.replace("type: runtime", "type: shell"),
        valid.replace("tool: python", "tool: cmd"),
        valid.replace("id: git", "id: python"),
        valid.replace(">=3.11.0", "nonsense"),
        valid.replace("    tool: python", "    command: whoami\n    tool: python"),
        valid.replace("id: python-workshop", "id: /home/private"),
        "schema_version: 1\nid: x\nname: x\nchecks: []".into(),
        "[".into(),
        "x".repeat(65537),
    ] {
        assert!(Profile::parse(&yaml).is_err(), "accepted invalid profile")
    }
}
#[test]
fn rejects_duplicate_yaml_keys() {
    assert!(
        Profile::parse("schema_version: 1\nschema_version: 2\nid: x\nname: x\nchecks: []").is_err()
    )
}

#[test]
fn rejects_tags_and_aliases() {
    let valid = include_str!("fixtures/runtime.yaml");
    for yaml in [
        valid.replace("name: \"Python delavnica\"", "name: !include /secret"),
        valid.replace("id: python-workshop", "id: &id python-workshop"),
        format!("{valid}\n---\n{valid}"),
    ] {
        assert!(Profile::parse(&yaml).is_err())
    }
}
#[tokio::test]
async fn version_metadata_cannot_leak() {
    let f = Fake::new(vec![(Invocation::Node, out("v22.1.0+secret-user", ""))]);
    let r = check_profile(&profile("node", true), &f, Platform::Unix).await;
    assert_eq!(r.results[0].installed_version.as_deref(), Some("22.1.0"));
    assert!(!serde_json::to_string(&r).unwrap().contains("secret-user"));
}

#[tokio::test]
async fn single_check_executes_only_selected_runtime() {
    let p = Profile::parse(include_str!("fixtures/runtime.yaml")).unwrap();
    let f = Fake::new(vec![(Invocation::Git, out("git version 2.53.0", ""))]);
    let result = preflight_core::engine::check_selected(
        &p,
        "git",
        &f,
        Platform::Windows,
        &preflight_core::host::SystemHost,
        false,
    )
    .await
    .unwrap();
    assert_eq!(result.check_id, "git");
    assert_eq!(result.status, Status::Pass);
    f.done();
}
#[tokio::test]
async fn single_check_rejects_unregistered_ids_before_spawning() {
    let p = profile("git", true);
    let f = Fake::new(vec![]);
    assert!(preflight_core::engine::check_selected(
        &p,
        "cmd /c whoami",
        &f,
        Platform::Windows,
        &preflight_core::host::SystemHost,
        false
    )
    .await
    .is_err());
    f.done();
}
