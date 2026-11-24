#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod classroom;
use classroom::{room_leave, room_load, room_share};
use preflight_core::{
    engine::{check_profile_with, Platform},
    host::SystemHost,
    process::SystemRunner,
    Profile, Report, Status,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
const BUNDLED: &[&str] = &[
    include_str!("../../../../profiles/python-workshop.yaml"),
    include_str!("../../../../profiles/java-workshop.yaml"),
    include_str!("../../../../profiles/web-workshop.yaml"),
];
#[derive(Debug, Serialize)]
struct CommandError {
    message_sl: &'static str,
}
impl From<preflight_core::Error> for CommandError {
    fn from(e: preflight_core::Error) -> Self {
        Self {
            message_sl: e.message_sl,
        }
    }
}
#[derive(Default)]
struct AppState {
    gate: Mutex<()>,
    last: Mutex<Option<Report>>,
    last_profile: Mutex<Option<String>>,
    room: Mutex<Option<classroom::Room>>,
}
fn profiles() -> Result<Vec<Profile>, CommandError> {
    BUNDLED
        .iter()
        .map(|y| Profile::parse(y).map_err(Into::into))
        .collect()
}
fn find_profile(id: &str) -> Result<Profile, CommandError> {
    profiles()?
        .into_iter()
        .find(|p| p.id() == id)
        .ok_or(CommandError {
            message_sl: "Profil ni na voljo. Izberi profil s seznama.",
        })
}
fn resolve_profile(id: &str, yaml: Option<String>) -> Result<Profile, CommandError> {
    match yaml {
        Some(y) => {
            let p = Profile::parse(&y)?;
            if p.id() != id {
                return Err(CommandError {
                    message_sl: "ID profila se ne ujema z izbiro.",
                });
            }
            Ok(p)
        }
        None => find_profile(id),
    }
}
#[tauri::command]
fn get_app_info() -> preflight_core::AppInfo {
    preflight_core::app_info()
}
#[tauri::command]
fn list_profiles() -> Result<Vec<Profile>, CommandError> {
    profiles()
}
#[tauri::command]
fn validate_profile(yaml: String) -> Result<Profile, CommandError> {
    Profile::parse(&yaml).map_err(Into::into)
}
#[tauri::command]
fn get_profile_yaml(
    profile_id: String,
    profile_yaml: Option<String>,
) -> Result<String, CommandError> {
    resolve_profile(&profile_id, profile_yaml)?
        .to_yaml()
        .map_err(Into::into)
}
#[tauri::command]
async fn run_checks(
    profile_id: String,
    profile_yaml: Option<String>,
    allow_network: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<Report, CommandError> {
    let _guard = state.gate.try_lock().map_err(|_| CommandError {
        message_sl: "Preverjanje že poteka. Počakaj na zaključek.",
    })?;
    let profile = resolve_profile(&profile_id, profile_yaml)?;
    *state.last.lock().await = None;
    *state.last_profile.lock().await = None;
    let report = check_profile_with(
        &profile,
        &SystemRunner,
        Platform::current(),
        &SystemHost,
        allow_network.unwrap_or(false),
    )
    .await;
    *state.last.lock().await = Some(report.clone());
    *state.last_profile.lock().await = Some(profile.to_yaml()?);
    Ok(report)
}
async fn save(content: String, extension: &str, filename: &str) -> Result<bool, CommandError> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .set_title("Shrani lokalno datoteko")
        .add_filter("Preflight", &[extension])
        .set_file_name(filename)
        .save_file()
        .await
    else {
        return Ok(false);
    };
    file.write(content.as_bytes())
        .await
        .map_err(|_| CommandError {
            message_sl: "Datoteke ni mogoče shraniti. Preveri izbrano mesto in dovoljenja.",
        })?;
    Ok(true)
}
#[tauri::command]
async fn export_profile(
    profile_id: String,
    profile_yaml: Option<String>,
) -> Result<bool, CommandError> {
    let profile = resolve_profile(&profile_id, profile_yaml)?;
    save(profile.to_yaml()?, "yaml", "preflight-profile.yaml").await
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExportFormat {
    Json,
    Text,
}
#[tauri::command]
async fn export_report(
    format: ExportFormat,
    state: tauri::State<'_, AppState>,
) -> Result<bool, CommandError> {
    let report = state.last.lock().await.clone().ok_or(CommandError {
        message_sl: "Najprej zaženi preverjanje.",
    })?;
    let (content, ext, name) = match format {
        ExportFormat::Json => (
            serde_json::to_string_pretty(&report).map_err(|_| CommandError {
                message_sl: "Izvoz ni uspel.",
            })?,
            "json",
            "preflight-results.json",
        ),
        ExportFormat::Text => {
            let mut s = String::from("Preflight — lokalni povzetek\n");
            for r in &report.results {
                let status = match r.status {
                    Status::Pass => "V REDU",
                    Status::Fail => "NAPAKA",
                    Status::Warning => "OPOZORILO",
                    Status::Unknown => "NEZNANO",
                };
                s.push_str(&format!(
                    "[{}] {} {}: {}\n{}\n",
                    status,
                    r.check_id,
                    r.installed_version.as_deref().unwrap_or(""),
                    r.message_sl,
                    r.suggestion_sl
                ))
            }
            (s, "txt", "preflight-results.txt")
        }
    };
    save(content, ext, name).await
}
fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            list_profiles,
            validate_profile,
            get_profile_yaml,
            run_checks,
            export_profile,
            export_report,
            room_load,
            room_share,
            room_leave
        ])
        .run(tauri::generate_context!())
        .expect("Namizne aplikacije ni bilo mogoče zagnati");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_profiles_are_valid() {
        let list = profiles().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].checks().len(), 6)
    }
    #[test]
    fn arbitrary_paths_are_rejected() {
        for id in [
            "../secret",
            "C:\\Users\\secret.yaml",
            "https://example.com/profile",
            "cmd",
        ] {
            assert!(find_profile(id).is_err())
        }
    }
    #[test]
    fn custom_profile_must_match_id() {
        assert!(resolve_profile("other", Some(BUNDLED[0].into())).is_err());
        assert!(resolve_profile("python-workshop", Some(BUNDLED[0].into())).is_ok())
    }
}
