use crate::{
    model::*,
    process::{Invocation, ProcessError, ProcessRunner},
    version,
};
#[derive(Clone, Copy)]
pub enum Platform {
    Windows,
    Unix,
}
impl Platform {
    pub fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Unix
        }
    }
}
pub async fn check_profile(
    profile: &Profile,
    runner: &impl ProcessRunner,
    platform: Platform,
) -> Report {
    let mut results = Vec::with_capacity(profile.checks().len());
    for check in profile.checks() {
        results.push(check_one(check, runner, platform).await)
    }
    Report::new(results)
}
async fn detect(
    tool: Tool,
    invocation: Invocation,
    runner: &impl ProcessRunner,
) -> Result<semver::Version, ReasonCode> {
    let out = runner.run(invocation).await.map_err(|e| match e {
        ProcessError::NotFound => ReasonCode::NotFound,
        ProcessError::Timeout => ReasonCode::Timeout,
        ProcessError::OutputLimit => ReasonCode::OutputLimit,
        ProcessError::Failed => ReasonCode::ProcessFailed,
    })?;
    version::parse(tool, &out.stdout)
        .or_else(|| version::parse(tool, &out.stderr))
        .ok_or(ReasonCode::UnrecognizedVersion)
}
async fn check_one(check: &Check, runner: &impl ProcessRunner, platform: Platform) -> CheckResult {
    let invocations: &[Invocation] = match check.tool {
        Tool::Python => match platform {
            Platform::Windows => &[Invocation::Python, Invocation::Py3],
            Platform::Unix => &[Invocation::Python3, Invocation::Python],
        },
        Tool::Git => &[Invocation::Git],
        Tool::Node => &[Invocation::Node],
        Tool::Java => &[Invocation::Java],
    };
    let mut detected = Err(ReasonCode::NotFound);
    for invocation in invocations {
        let attempt = detect(check.tool, *invocation, runner).await;
        if attempt.is_ok() {
            detected = attempt;
            break;
        }
        // Preserve a meaningful error if a fallback is merely absent.
        if attempt != Err(ReasonCode::NotFound) || detected == Err(ReasonCode::NotFound) {
            detected = attempt
        }
    }
    let version = match detected {
        Ok(v) => v,
        Err(reason) => return result(check, reason, None),
    };
    let requirement = match semver::VersionReq::parse(&check.version) {
        Ok(r) => r,
        Err(_) => return result(check, ReasonCode::InvalidProfile, None),
    };
    if !requirement.matches(&version) {
        return result(
            check,
            ReasonCode::VersionMismatch,
            Some(version.to_string()),
        );
    }
    if check.tool == Tool::Java {
        match detect(Tool::Java, Invocation::Javac, runner).await {
            Err(ReasonCode::NotFound) => {
                return result(check, ReasonCode::JdkMissing, Some(version.to_string()))
            }
            Err(reason) => return result(check, reason, Some(version.to_string())),
            Ok(compiler) if !requirement.matches(&compiler) || compiler.major != version.major => {
                return result(check, ReasonCode::JdkMismatch, Some(version.to_string()))
            }
            Ok(_) => {}
        }
    }
    result(check, ReasonCode::Satisfied, Some(version.to_string()))
}
fn result(
    check: &Check,
    reason_code: ReasonCode,
    installed_version: Option<String>,
) -> CheckResult {
    let (status, message_sl, suggestion_sl) = match reason_code {
        ReasonCode::Satisfied => (
            Status::Pass,
            "Zahtevana različica je na voljo.",
            "Okolje je pripravljeno.",
        ),
        ReasonCode::NotFound => (
            Status::Fail,
            "Programa ni mogoče najti na PATH.",
            "Namesti zahtevano orodje ali popravi PATH in znova zaženi Preflight.",
        ),
        ReasonCode::VersionMismatch => (
            Status::Fail,
            "Nameščena različica ne ustreza profilu.",
            "Namesti različico, ki ustreza zahtevi profila, in preveri PATH.",
        ),
        ReasonCode::JdkMissing => (
            Status::Fail,
            "Java je na voljo, prevajalnik JDK pa manjka.",
            "Namesti JDK in dodaj njegov imenik bin na PATH.",
        ),
        ReasonCode::JdkMismatch => (
            Status::Fail,
            "Različici Java in prevajalnika JDK nista ustrezni.",
            "Na PATH izberi skladen JDK, ki ustreza profilu.",
        ),
        ReasonCode::Timeout => (
            Status::Unknown,
            "Program se ni odzval v časovni omejitvi.",
            "Preveri delovanje orodja in ponovi preverjanje.",
        ),
        ReasonCode::OutputLimit => (
            Status::Unknown,
            "Program je vrnil preveč izhodnih podatkov.",
            "Preveri namestitev orodja; izhod ni bil shranjen.",
        ),
        ReasonCode::UnrecognizedVersion => (
            Status::Unknown,
            "Različice ni mogoče zanesljivo prepoznati.",
            "Preveri stabilno različico orodja in njegovo namestitev.",
        ),
        _ => (
            Status::Unknown,
            "Programa ni bilo mogoče uspešno preveriti.",
            "Preveri dovoljenja in namestitev ter poskusi znova.",
        ),
    };
    let status = if !check.required && status == Status::Fail {
        Status::Warning
    } else {
        status
    };
    CheckResult {
        check_id: check.id.clone(),
        status,
        required: check.required,
        installed_version,
        reason_code,
        message_sl,
        suggestion_sl,
    }
}
