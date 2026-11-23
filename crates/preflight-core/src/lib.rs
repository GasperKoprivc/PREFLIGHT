use serde::Serialize;
#[derive(Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "Preflight",
        version: env!("CARGO_PKG_VERSION"),
    }
}
