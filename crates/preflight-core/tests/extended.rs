use preflight_core::{
    engine::{check_profile_with, Platform},
    host::HostChecks,
    network::*,
    process::*,
    *,
};
use std::{collections::VecDeque, net::SocketAddr, sync::Mutex};
struct HostFake {
    disk: Result<u64, ReasonCode>,
    network: ReasonCode,
}
impl HostChecks for HostFake {
    async fn free_bytes(&self) -> Result<u64, ReasonCode> {
        self.disk
    }
    async fn https(&self, _: &str) -> ReasonCode {
        self.network
    }
}
struct Runner(Result<Output, ProcessError>);
impl ProcessRunner for Runner {
    async fn run(&self, i: Invocation) -> Result<Output, ProcessError> {
        assert_eq!(i, Invocation::CodeExtensions);
        self.0.clone()
    }
}
fn profile(check: &str) -> Profile {
    Profile::parse(&format!("schema_version: 1\nid: test\nname: Test\nchecks:\n  - id: test\n    required: true\n{check}")).unwrap()
}
#[tokio::test]
async fn disk_results() {
    for (bytes, reason) in [
        (Ok(5_000_000_000), ReasonCode::DiskAvailable),
        (Ok(4_999_999_999), ReasonCode::DiskLow),
        (
            Err(ReasonCode::DiskUnavailable),
            ReasonCode::DiskUnavailable,
        ),
    ] {
        let p = profile("    type: disk_space\n    min_free_gb: 5");
        let r = check_profile_with(
            &p,
            &Runner(Err(ProcessError::Failed)),
            Platform::Unix,
            &HostFake {
                disk: bytes,
                network: ReasonCode::InternalError,
            },
            false,
        )
        .await;
        assert_eq!(r.results[0].reason_code, reason)
    }
}
#[tokio::test]
async fn extension_results() {
    for (out, reason) in [
        (
            Ok(Output {
                stdout: "ms-python.python\nredhat.java\n".into(),
                stderr: "".into(),
            }),
            ReasonCode::ExtensionPresent,
        ),
        (
            Ok(Output {
                stdout: "redhat.java\n".into(),
                stderr: "".into(),
            }),
            ReasonCode::ExtensionMissing,
        ),
        (Err(ProcessError::NotFound), ReasonCode::VscodeUnavailable),
        (Err(ProcessError::Timeout), ReasonCode::Timeout),
        (Err(ProcessError::OutputLimit), ReasonCode::OutputLimit),
    ] {
        let p = profile("    type: vscode_extension\n    extension: ms-python.python");
        let r = check_profile_with(
            &p,
            &Runner(out),
            Platform::Windows,
            &HostFake {
                disk: Ok(0),
                network: ReasonCode::InternalError,
            },
            false,
        )
        .await;
        assert_eq!(r.results[0].reason_code, reason);
        let j = serde_json::to_string(&r).unwrap();
        assert!(!j.contains("redhat.java"));
    }
}
#[tokio::test]
async fn offline_skips_network() {
    let p = profile("    type: https\n    url: https://example.com");
    let r = check_profile_with(
        &p,
        &Runner(Err(ProcessError::Failed)),
        Platform::Unix,
        &HostFake {
            disk: Ok(0),
            network: ReasonCode::HttpsReachable,
        },
        false,
    )
    .await;
    assert_eq!(r.results[0].reason_code, ReasonCode::NetworkSkipped);
    assert_eq!(r.exit_code(), 1)
}
#[test]
fn invalid_variants() {
    for check in [
        "type: disk_space\n    min_free_gb: -1",
        "type: disk_space\n    min_free_gb: .nan",
        "type: https\n    url: http://example.com",
        "type: https\n    url: https://127.0.0.1",
        "type: vscode_extension\n    extension: '../secret'",
        "type: disk_space\n    min_free_gb: 3\n    tool: node",
        "type: runtime\n    tool: python",
        "type: https\n    url: https://example.com\n    command: whoami",
    ] {
        assert!(Profile::parse(&format!(
            "schema_version: 1\nid: x\nname: X\nchecks:\n - id: x\n   required: true\n   {}",
            check.replace("    ", "   ")
        ))
        .is_err())
    }
}
#[test]
fn all_examples_roundtrip() {
    for yaml in [
        include_str!("../../../profiles/python-workshop.yaml"),
        include_str!("../../../profiles/java-workshop.yaml"),
        include_str!("../../../profiles/web-workshop.yaml"),
    ] {
        let p = Profile::parse(yaml).unwrap();
        let q = Profile::parse(&p.to_yaml().unwrap()).unwrap();
        assert_eq!(p.id(), q.id());
        assert_eq!(p.checks().len(), q.checks().len());
    }
}
#[test]
fn blocks_nonpublic_urls() {
    for u in [
        "http://example.com",
        "https://localhost",
        "https://localhost./",
        "https://x.localhost",
        "https://127.1",
        "https://2130706433",
        "https://0x7f000001",
        "https://10.0.0.1",
        "https://169.254.169.254",
        "https://100.100.100.200",
        "https://192.168.1.1",
        "https://[::1]",
        "https://[::ffff:127.0.0.1]",
        "https://[fc00::1]",
        "https://[fe80::1]",
        "https://[2001:db8::1]",
        "https://[2002:7f00:1::]",
        "https://user:pass@example.com",
        "https://example.com:8443",
        "https://example.com/#secret",
        "file:///etc/passwd",
    ] {
        assert!(validate_url(u).is_err(), "accepted {u}")
    }
}
#[test]
fn permits_public_urls() {
    for u in [
        "https://example.com",
        "https://1.1.1.1",
        "https://[2606:4700:4700::1111]/",
    ] {
        assert!(validate_url(u).is_ok())
    }
}
struct NetFake {
    dns: Mutex<VecDeque<Vec<SocketAddr>>>,
    hops: Mutex<VecDeque<Hop>>,
    calls: Mutex<Vec<Vec<SocketAddr>>>,
}
impl NetFake {
    fn new(dns: Vec<Vec<&str>>, hops: Vec<Hop>) -> Self {
        Self {
            dns: Mutex::new(
                dns.into_iter()
                    .map(|v| v.into_iter().map(|s| s.parse().unwrap()).collect())
                    .collect(),
            ),
            hops: Mutex::new(hops.into()),
            calls: Mutex::new(Vec::new()),
        }
    }
}
impl Transport for NetFake {
    async fn resolve(&self, _: &str) -> Result<Vec<SocketAddr>, ReasonCode> {
        Ok(self
            .dns
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected DNS"))
    }
    async fn head(&self, _: &url::Url, pinned: &[SocketAddr]) -> Result<Hop, ReasonCode> {
        assert!(pinned.iter().all(|a| public_ip(a.ip())));
        self.calls.lock().unwrap().push(pinned.to_vec());
        Ok(self
            .hops
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected request"))
    }
}
fn hop(status: u16, location: Option<&str>) -> Hop {
    Hop {
        status,
        location: location.map(str::to_owned),
    }
}
#[tokio::test]
async fn rejects_mixed_dns_before_request() {
    let n = NetFake::new(vec![vec!["1.1.1.1:443", "127.0.0.1:443"]], vec![]);
    assert_eq!(
        check_https("https://example.com", &n).await,
        ReasonCode::UnsafeUrl
    );
    assert!(n.calls.lock().unwrap().is_empty())
}
#[tokio::test]
async fn pins_validated_dns() {
    let n = NetFake::new(vec![vec!["1.1.1.1:443"]], vec![hop(200, None)]);
    assert_eq!(
        check_https("https://example.com", &n).await,
        ReasonCode::HttpsReachable
    );
    assert_eq!(
        n.calls.lock().unwrap()[0],
        vec!["1.1.1.1:443".parse::<SocketAddr>().unwrap()]
    )
}
#[tokio::test]
async fn redirect_to_metadata_is_blocked() {
    let n = NetFake::new(
        vec![vec!["1.1.1.1:443"]],
        vec![hop(302, Some("https://169.254.169.254/"))],
    );
    assert_eq!(
        check_https("https://example.com", &n).await,
        ReasonCode::UnsafeUrl
    );
    assert_eq!(n.calls.lock().unwrap().len(), 1)
}
#[tokio::test]
async fn dns_rebinding_on_redirect_is_blocked() {
    let n = NetFake::new(
        vec![vec!["1.1.1.1:443"], vec!["10.0.0.1:443"]],
        vec![hop(302, Some("/next"))],
    );
    assert_eq!(
        check_https("https://example.com", &n).await,
        ReasonCode::UnsafeUrl
    );
    assert_eq!(n.calls.lock().unwrap().len(), 1)
}
#[tokio::test]
async fn redirect_limit() {
    let n = NetFake::new(
        vec![vec!["1.1.1.1:443"]; 4],
        (0..4).map(|_| hop(302, Some("/next"))).collect(),
    );
    assert_eq!(
        check_https("https://example.com", &n).await,
        ReasonCode::RedirectLimit
    )
}
#[tokio::test]
async fn non_success_is_not_pass() {
    let n = NetFake::new(vec![vec!["1.1.1.1:443"]], vec![hop(405, None)]);
    assert_eq!(
        check_https("https://example.com", &n).await,
        ReasonCode::HttpsStatus
    )
}
