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
    check_profile_with(profile, runner, platform, &crate::host::SystemHost, true).await
}
pub async fn check_profile_with(
    profile: &Profile,
    runner: &impl ProcessRunner,
    platform: Platform,
    host: &impl crate::host::HostChecks,
    allow_network: bool,
) -> Report {
    let mut results = Vec::with_capacity(profile.checks().len());
    for check in profile.checks() {
        results.push(match check.check_type {
            CheckType::Runtime => check_one(check, runner, platform).await,
            _ => crate::host::check_extra(check, runner, host, allow_network).await,
        })
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
    let tool = check.tool.expect("validated runtime tool");
    let invocations: &[Invocation] = match tool {
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
        let attempt = detect(tool, *invocation, runner).await;
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
    let requirement = match semver::VersionReq::parse(
        check
            .version
            .as_deref()
            .expect("validated runtime constraint"),
    ) {
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
    if tool == Tool::Java {
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
pub(crate) fn result(
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
        ReasonCode::ExtensionPresent => (Status::Pass,"Razširitev VS Code je nameščena.","Razširitev je pripravljena."),
        ReasonCode::ExtensionMissing => (Status::Fail,"Zahtevana razširitev VS Code manjka.","V VS Code odpri Extensions, poišči ID iz profila in namesti razširitev."),
        ReasonCode::VscodeUnavailable => (Status::Unknown,"Seznama razširitev VS Code ni mogoče pridobiti.","Preveri VS Code CLI in privzeti profil. Ročno preveri razširitev v zavihku Extensions."),
        ReasonCode::DiskAvailable => (Status::Pass,"Na disku je dovolj prostega prostora.","Zahteva za prostor je izpolnjena."),
        ReasonCode::DiskLow => (Status::Fail,"Na disku ni dovolj prostega prostora.","Sprosti prostor na disku delovne mape in ponovi preverjanje."),
        ReasonCode::DiskUnavailable => (Status::Unknown,"Prostega prostora ni mogoče preveriti.","Ročno preveri prostor na disku delovne mape."),
        ReasonCode::HttpsReachable => (Status::Pass,"HTTPS naslov je dosegljiv.","Varna povezava je uspela."),
        ReasonCode::HttpsStatus => (Status::Warning,"Strežnik ni vrnil uspešnega odgovora na HEAD.","Preveri dosegljivost v brskalniku; nekateri strežniki HEAD ne podpirajo."),
        ReasonCode::NetworkSkipped => (Status::Unknown,"Spletno preverjanje je izključeno.","Če želiš preveriti povezavo, dovoli HTTPS preverjanja in zaženi znova."),
        ReasonCode::UnsafeUrl => (Status::Unknown,"Naslov ali preusmeritev je blokirana zaradi varnosti.","Uporabi javni HTTPS naslov brez uporabniških podatkov na vratih 443."),
        ReasonCode::RedirectLimit => (Status::Unknown,"Preveč spletnih preusmeritev.","Preveri končni javni HTTPS naslov v profilu."),
        ReasonCode::NetworkUnavailable => (Status::Unknown,"HTTPS povezave ni mogoče potrditi.","Preveri internetno povezavo, DNS in veljavnost certifikata."),
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
