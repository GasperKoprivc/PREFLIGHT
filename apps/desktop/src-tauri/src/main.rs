#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[tauri::command]
fn get_app_info() -> preflight_core::AppInfo {
    preflight_core::app_info()
}
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_app_info])
        .run(tauri::generate_context!())
        .expect("Namizne aplikacije ni bilo mogoče zagnati");
}
