#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use preflight_core::{engine::Platform, process::SystemRunner, Profile, Report};
use serde::Serialize;
use tokio::sync::Mutex;

const PYTHON_PROFILE: &str = include_str!("../../../../profiles/python-workshop.yaml");
#[derive(Serialize)]
struct CommandError {
    message_sl: &'static str,
}
fn profiles() -> Result<Vec<Profile>, CommandError> {
    Ok(vec![Profile::parse(PYTHON_PROFILE).map_err(|_| {
        CommandError {
            message_sl: "Vgrajeni profil ni veljaven.",
        }
    })?])
}
fn find_profile(id: &str) -> Result<Profile, CommandError> {
    profiles()?
        .into_iter()
        .find(|p| p.id() == id)
        .ok_or(CommandError {
            message_sl: "Profil ni na voljo. Izberi profil s seznama.",
        })
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
async fn run_checks(
    profile_id: String,
    state: tauri::State<'_, Mutex<()>>,
) -> Result<Report, CommandError> {
    let _guard = state.try_lock().map_err(|_| CommandError {
        message_sl: "Preverjanje že poteka. Počakaj na zaključek.",
    })?;
    let profile = find_profile(&profile_id)?;
    Ok(preflight_core::check_profile(&profile, &SystemRunner, Platform::current()).await)
}
fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(()))
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            list_profiles,
            run_checks
        ])
        .run(tauri::generate_context!())
        .expect("Namizne aplikacije ni bilo mogoče zagnati");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_profiles_are_valid() {
        let list = profiles().ok().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].checks().len(), 3);
        assert!(find_profile("python-workshop").is_ok())
    }
    #[test]
    fn arbitrary_profile_paths_are_rejected() {
        for id in [
            "../secret",
            "C:\\Users\\secret.yaml",
            "https://example.com/profile",
            "cmd",
        ] {
            assert!(find_profile(id).is_err())
        }
    }
}
