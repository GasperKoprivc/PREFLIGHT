use crate::{Check, CheckResult, ReasonCode, Tool};
use serde::Serialize;
use std::future::Future;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallInvocation {
    WingetVersion,
    GitWindows2561,
}
impl InstallInvocation {
    pub(crate) fn command(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::WingetVersion => ("winget", &["--version"]),
            Self::GitWindows2561 => (
                "winget",
                &[
                    "install",
                    "--id",
                    "Git.Git",
                    "--exact",
                    "--source",
                    "winget",
                    "--version",
                    "2.56.1",
                    "--architecture",
                    "x64",
                    "--silent",
                    "--disable-interactivity",
                ],
            ),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct FixAction {
    pub id: &'static str,
    pub name_sl: &'static str,
    pub manager: &'static str,
    pub package_id: &'static str,
    pub version: &'static str,
    pub command: &'static str,
    pub effects_sl: &'static str,
    pub permissions_sl: &'static str,
    pub license_url: &'static str,
}
pub fn git_action(check: &Check, result: &CheckResult, os: &str, arch: &str) -> Option<FixAction> {
    if os != "windows"
        || arch != "x86_64"
        || check.tool != Some(Tool::Git)
        || check.id != result.check_id
        || !matches!(
            result.reason_code,
            ReasonCode::NotFound | ReasonCode::VersionMismatch
        )
    {
        return None;
    }
    let requirement = semver::VersionReq::parse(check.version.as_deref()?).ok()?;
    if !requirement.matches(&semver::Version::new(2, 56, 1)) {
        return None;
    }
    Some(FixAction{id:"git-windows-2.56.1",name_sl:"Git za Windows",manager:"winget",package_id:"Git.Git",version:"2.56.1",command:"winget install --id Git.Git --exact --source winget --version 2.56.1 --architecture x64 --silent --disable-interactivity",effects_sl:"Prenese in namesti oziroma posodobi Git za Windows 2.56.1 iz vira winget. Spremeni namestitev Git in lahko PATH. Winget preveri hash iz manifesta. Brez samodejnega ponovnega zagona ali sprejemanja licenčnih pogojev.",permissions_sl:"Namestitveni program lahko zahteva administratorsko potrditev Windows. Če upravljalnik, dovoljenja ali sprejeti pogoji vira niso na voljo, uporabi ročni vodič. Preflight ne potrjuje sistemskih pozivov.",license_url:"https://git-scm.com/about/free-and-open-source"})
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallError {
    ManagerUnavailable,
    Failed,
    Timeout,
    OutputLimit,
}
pub trait InstallRunner: Sync {
    fn run_install(
        &self,
        invocation: InstallInvocation,
    ) -> impl Future<Output = Result<(), InstallError>> + Send;
}
pub async fn install_git(runner: &impl InstallRunner) -> Result<(), InstallError> {
    runner
        .run_install(InstallInvocation::WingetVersion)
        .await
        .map_err(|_| InstallError::ManagerUnavailable)?;
    runner.run_install(InstallInvocation::GitWindows2561).await
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Profile;
    use std::sync::Mutex;
    fn profile(version: &str) -> Profile {
        Profile::parse(&format!("schema_version: 1\nid: git\nname: Git\nchecks:\n - id: git\n   type: runtime\n   tool: git\n   version: '{version}'\n   required: true")).unwrap()
    }
    fn missing() -> CheckResult {
        CheckResult {
            check_id: "git".into(),
            status: crate::Status::Fail,
            required: true,
            installed_version: None,
            reason_code: ReasonCode::NotFound,
            message_sl: "",
            suggestion_sl: "",
        }
    }
    #[test]
    fn fixed_catalog_and_platform() {
        let p = profile(">=2.40.0");
        assert!(git_action(&p.checks()[0], &missing(), "windows", "x86_64").is_some());
        for os in ["macos", "linux", "other"] {
            assert!(git_action(&p.checks()[0], &missing(), os, "x86_64").is_none())
        }
        assert!(git_action(&p.checks()[0], &missing(), "windows", "aarch64").is_none());
        assert!(git_action(
            &profile("<2.50.0").checks()[0],
            &missing(),
            "windows",
            "x86_64"
        )
        .is_none());
        let mut r = missing();
        r.reason_code = ReasonCode::Timeout;
        assert!(git_action(&p.checks()[0], &r, "windows", "x86_64").is_none())
    }
    struct Fake {
        calls: Mutex<Vec<InstallInvocation>>,
        manager: bool,
        result: Result<(), InstallError>,
    }
    impl InstallRunner for Fake {
        async fn run_install(&self, i: InstallInvocation) -> Result<(), InstallError> {
            self.calls.lock().unwrap().push(i);
            if i == InstallInvocation::WingetVersion {
                if self.manager {
                    Ok(())
                } else {
                    Err(InstallError::Failed)
                }
            } else {
                self.result
            }
        }
    }
    #[tokio::test]
    async fn manager_required() {
        let f = Fake {
            calls: Mutex::new(vec![]),
            manager: false,
            result: Ok(()),
        };
        assert_eq!(install_git(&f).await, Err(InstallError::ManagerUnavailable));
        assert_eq!(
            *f.calls.lock().unwrap(),
            vec![InstallInvocation::WingetVersion]
        )
    }
    #[tokio::test]
    async fn success_and_errors() {
        for result in [
            Ok(()),
            Err(InstallError::Failed),
            Err(InstallError::Timeout),
            Err(InstallError::OutputLimit),
        ] {
            let f = Fake {
                calls: Mutex::new(vec![]),
                manager: true,
                result,
            };
            assert_eq!(install_git(&f).await, result);
            assert_eq!(
                *f.calls.lock().unwrap(),
                vec![
                    InstallInvocation::WingetVersion,
                    InstallInvocation::GitWindows2561
                ]
            )
        }
    }
    #[test]
    fn no_shell_or_unpinned_arguments() {
        let (program, args) = InstallInvocation::GitWindows2561.command();
        assert_eq!(program, "winget");
        assert!(args.windows(2).any(|a| a == ["--version", "2.56.1"]));
        assert!(args.windows(2).any(|a| a == ["--id", "Git.Git"]));
        assert!(!args.iter().any(|a| a.contains(';')
            || a.contains('|')
            || a.contains("accept-")
            || a.contains("override")))
    }
}
