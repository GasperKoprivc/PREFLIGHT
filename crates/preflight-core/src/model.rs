use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_PROFILE_BYTES: usize = 64 * 1024;
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    Python,
    Git,
    Node,
    Java,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckType {
    Runtime,
    VscodeExtension,
    DiskSpace,
    Https,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub id: String,
    #[serde(rename = "type")]
    pub check_type: CheckType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<Tool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_free_gb: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub required: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfileData {
    schema_version: u32,
    id: String,
    name: String,
    checks: Vec<Check>,
}
/// Only validated profiles can be passed to the engine.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Profile(ProfileData);
impl Profile {
    pub fn parse(yaml: &str) -> Result<Self, Error> {
        if yaml.len() > MAX_PROFILE_BYTES {
            return Err(Error::new(ReasonCode::InvalidProfile));
        }
        let data: ProfileData =
            serde_saphyr::from_str_with_options(yaml, serde_saphyr::options! { reject_unsupported_tags: true, budget: serde_saphyr::budget! { max_depth: 8, max_nodes: 1024, max_events: 2048, max_aliases: 0, max_anchors: 0, max_documents: 1, max_total_scalar_bytes: MAX_PROFILE_BYTES } }).map_err(|_| Error::new(ReasonCode::InvalidProfile))?;
        let mut ids = HashSet::new();
        if data.schema_version != 1
            || !valid_id(&data.id)
            || data.name.trim().is_empty()
            || data.name.chars().count() > 100
            || data.name.chars().any(char::is_control)
            || data.checks.is_empty()
            || data.checks.len() > 32
        {
            return Err(Error::new(ReasonCode::InvalidProfile));
        }
        for check in &data.checks {
            if !valid_id(&check.id) || !ids.insert(&check.id) || !check.valid() {
                return Err(Error::new(ReasonCode::InvalidProfile));
            }
        }
        Ok(Self(data))
    }
    pub fn to_yaml(&self) -> Result<String, Error> {
        serde_saphyr::to_string(&self.0).map_err(|_| Error::new(ReasonCode::InternalError))
    }
    pub fn has_network(&self) -> bool {
        self.0
            .checks
            .iter()
            .any(|c| matches!(c.check_type, CheckType::Https))
    }
    pub fn checks(&self) -> &[Check] {
        &self.0.checks
    }
    pub fn name(&self) -> &str {
        &self.0.name
    }
    pub fn id(&self) -> &str {
        &self.0.id
    }
}
fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    Warning,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    Satisfied,
    VersionMismatch,
    NotFound,
    Timeout,
    OutputLimit,
    ProcessFailed,
    UnrecognizedVersion,
    JdkMissing,
    JdkMismatch,
    InvalidProfile,
    ProfileUnreadable,
    InternalError,
    ExtensionPresent,
    ExtensionMissing,
    VscodeUnavailable,
    DiskAvailable,
    DiskLow,
    DiskUnavailable,
    HttpsReachable,
    HttpsStatus,
    NetworkUnavailable,
    NetworkSkipped,
    UnsafeUrl,
    RedirectLimit,
}
#[derive(Debug, Clone, Serialize)]
pub struct Error {
    pub schema_version: u32,
    pub reason_code: ReasonCode,
    pub message_sl: &'static str,
}
impl Error {
    pub fn new(reason_code: ReasonCode) -> Self {
        Self { schema_version: 1, reason_code, message_sl: match reason_code {
  ReasonCode::InvalidProfile => "Profil ni veljaven. Preveri shemo 1, enolične ID-je, dovoljene vrste in omejitve različic.",
  ReasonCode::ProfileUnreadable => "Profila ni mogoče prebrati. Preveri datoteko, dovoljenja in velikost do 64 KiB.",
  _ => "Preverjanja ni bilo mogoče izvesti.",
 } }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message_sl)
    }
}
impl std::error::Error for Error {}
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub check_id: String,
    pub status: Status,
    pub required: bool,
    pub installed_version: Option<String>,
    pub reason_code: ReasonCode,
    pub message_sl: &'static str,
    pub suggestion_sl: &'static str,
}
#[derive(Debug, Clone, Serialize, Default)]
pub struct Summary {
    pub total: usize,
    pub pass: usize,
    pub fail: usize,
    pub warning: usize,
    pub unknown: usize,
    pub ready: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub results: Vec<CheckResult>,
    pub summary: Summary,
}
impl Report {
    pub fn new(results: Vec<CheckResult>) -> Self {
        let mut summary = Summary {
            total: results.len(),
            ready: true,
            ..Summary::default()
        };
        for r in &results {
            match r.status {
                Status::Pass => summary.pass += 1,
                Status::Fail => summary.fail += 1,
                Status::Warning => summary.warning += 1,
                Status::Unknown => summary.unknown += 1,
            }
            if r.required && r.status != Status::Pass {
                summary.ready = false;
            }
        }
        Self {
            schema_version: 1,
            results,
            summary,
        }
    }
    pub fn exit_code(&self) -> u8 {
        if self.summary.ready {
            0
        } else {
            1
        }
    }
}

impl Check {
    fn valid(&self) -> bool {
        match self.check_type {
            CheckType::Runtime => {
                self.tool.is_some()
                    && self
                        .version
                        .as_ref()
                        .is_some_and(|v| v.len() <= 128 && semver::VersionReq::parse(v).is_ok())
                    && self.extension.is_none()
                    && self.min_free_gb.is_none()
                    && self.url.is_none()
            }
            CheckType::VscodeExtension => {
                self.tool.is_none()
                    && self.version.is_none()
                    && self.min_free_gb.is_none()
                    && self.url.is_none()
                    && self.extension.as_ref().is_some_and(|s| {
                        let parts: Vec<_> = s.split('.').collect();
                        s.len() <= 128
                            && parts.len() == 2
                            && parts.iter().all(|p| {
                                !p.is_empty()
                                    && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                            })
                    })
            }
            CheckType::DiskSpace => {
                self.tool.is_none()
                    && self.version.is_none()
                    && self.extension.is_none()
                    && self.url.is_none()
                    && self
                        .min_free_gb
                        .is_some_and(|v| v.is_finite() && v > 0.0 && v <= 1000000.0)
            }
            CheckType::Https => {
                self.tool.is_none()
                    && self.version.is_none()
                    && self.extension.is_none()
                    && self.min_free_gb.is_none()
                    && self
                        .url
                        .as_ref()
                        .is_some_and(|u| crate::network::validate_url(u).is_ok())
            }
        }
    }
}
