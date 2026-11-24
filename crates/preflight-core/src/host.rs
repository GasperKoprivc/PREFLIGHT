use crate::{
    engine::result,
    model::*,
    process::{Invocation, ProcessError, ProcessRunner},
};
use std::future::Future;
pub trait HostChecks: Sync {
    fn free_bytes(&self) -> impl Future<Output = Result<u64, ReasonCode>> + Send;
    fn https(&self, url: &str) -> impl Future<Output = ReasonCode> + Send;
}
pub struct SystemHost;
impl HostChecks for SystemHost {
    async fn free_bytes(&self) -> Result<u64, ReasonCode> {
        tokio::task::spawn_blocking(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| fs4::available_space(p).ok())
                .ok_or(ReasonCode::DiskUnavailable)
        })
        .await
        .unwrap_or(Err(ReasonCode::DiskUnavailable))
    }
    async fn https(&self, url: &str) -> ReasonCode {
        crate::network::check_https(url, &crate::network::SystemTransport).await
    }
}
pub async fn check_extra(
    check: &Check,
    runner: &impl ProcessRunner,
    host: &impl HostChecks,
    allow_network: bool,
) -> CheckResult {
    let reason = match check.check_type {
        CheckType::VscodeExtension => match runner.run(Invocation::CodeExtensions).await {
            Ok(output) => {
                let wanted = check.extension.as_deref().unwrap_or("");
                if output
                    .stdout
                    .lines()
                    .any(|l| l.trim().eq_ignore_ascii_case(wanted))
                {
                    ReasonCode::ExtensionPresent
                } else {
                    ReasonCode::ExtensionMissing
                }
            }
            Err(ProcessError::Timeout) => ReasonCode::Timeout,
            Err(ProcessError::OutputLimit) => ReasonCode::OutputLimit,
            _ => ReasonCode::VscodeUnavailable,
        },
        CheckType::DiskSpace => match host.free_bytes().await {
            Ok(bytes)
                if (bytes as f64)
                    >= check.min_free_gb.unwrap_or(f64::INFINITY) * 1_000_000_000.0 =>
            {
                ReasonCode::DiskAvailable
            }
            Ok(_) => ReasonCode::DiskLow,
            Err(e) => e,
        },
        CheckType::Https => {
            if allow_network {
                host.https(check.url.as_deref().unwrap_or("")).await
            } else {
                ReasonCode::NetworkSkipped
            }
        }
        _ => ReasonCode::InternalError,
    };
    result(check, reason, None)
}
