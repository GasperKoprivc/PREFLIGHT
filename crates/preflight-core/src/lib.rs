pub mod engine;
pub mod model;
pub mod process;
pub mod version;
pub use engine::check_profile;
pub use model::{Check, CheckResult, Error, Profile, ReasonCode, Report, Status, Summary, Tool};
use serde::Serialize;
#[derive(Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub platform: &'static str,
    pub version: &'static str,
}
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "Preflight",
        platform: std::env::consts::OS,
        version: env!("CARGO_PKG_VERSION"),
    }
}

pub mod host;
pub mod network;
