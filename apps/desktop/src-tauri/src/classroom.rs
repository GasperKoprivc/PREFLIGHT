use super::{AppState, CommandError};
use preflight_core::{Profile, Report};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
fn error() -> CommandError {
    CommandError{message_sl:"Povezava s sobo ni uspela. Preveri naslov, kodo in povezavo; pred izhodom poskusi znova izbrisati sejo."}
}
pub struct Room {
    base: String,
    code: String,
    profile: Profile,
    expires: i64,
    session: Option<Session>,
}
#[derive(Deserialize)]
struct Session {
    session_id: String,
    session_token: String,
}
#[derive(Serialize)]
pub struct RoomView {
    pub code: String,
    pub profile: Profile,
    pub yaml: String,
    pub expires_at: i64,
    pub shared: bool,
}
impl Room {
    fn view(&self) -> Result<RoomView, CommandError> {
        Ok(RoomView {
            code: self.code.clone(),
            profile: self.profile.clone(),
            yaml: self.profile.to_yaml()?,
            expires_at: self.expires,
            shared: self.session.is_some(),
        })
    }
}
pub fn server_base(input: &str) -> Result<String, CommandError> {
    let u = url::Url::parse(input).map_err(|_| error())?;
    let local = matches!(u.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
    if !(u.scheme() == "https" || (u.scheme() == "http" && local))
        || u.host_str().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
        || u.path() != "/"
    {
        return Err(error());
    }
    Ok(u.origin().ascii_serialization())
}
fn client() -> Result<Client, CommandError> {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .build()
        .map_err(|_| error())
}
async fn request(
    base: &str,
    path: &str,
    method: Method,
    token: Option<&str>,
    body: Option<Value>,
) -> Result<Value, CommandError> {
    let mut request = client()?.request(method, format!("{base}/api/v1{path}"));
    if let Some(t) = token {
        request = request.bearer_auth(t)
    }
    if let Some(b) = body {
        request = request.json(&b)
    }
    let mut response = request.send().await.map_err(|_| error())?;
    if !response.status().is_success() {
        return Err(CommandError {
            message_sl: match response.status().as_u16() {
                404 => "Soba je potekla ali je bila izbrisana.",
                401 => "Dostop do seje ni dovoljen.",
                429 => "Preveč zahtev. Poskusi čez minuto.",
                _ => "Zahteva sobi ni uspela. Preveri profil in povezavo.",
            },
        });
    }
    if response.status().as_u16() == 204 {
        return Ok(Value::Null);
    }
    let mut data = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| error())? {
        if data.len() + chunk.len() > 128 * 1024 {
            return Err(error());
        }
        data.extend_from_slice(&chunk)
    }
    serde_json::from_slice(&data).map_err(|_| error())
}
pub fn payload(report: &Report) -> Value {
    json!({"schema_version":1,"consent":true,"results":report.results.iter().map(|r|json!({"check_id":r.check_id,"status":r.status,"installed_version":r.installed_version,"reason_code":r.reason_code})).collect::<Vec<_>>()})
}
#[tauri::command]
pub async fn room_load(
    server: String,
    code: String,
    state: tauri::State<'_, AppState>,
) -> Result<RoomView, CommandError> {
    let _gate = state.gate.lock().await;
    if state.room.lock().await.is_some() {
        return Err(CommandError {
            message_sl: "Najprej zapusti trenutno sobo.",
        });
    }
    let base = server_base(&server)?;
    let code = code.trim().to_ascii_uppercase();
    if code.len() != 8
        || !code
            .bytes()
            .all(|b| b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789".contains(&b))
    {
        return Err(error());
    }
    let data = request(
        &base,
        &format!("/rooms/{code}/profile"),
        Method::GET,
        None,
        None,
    )
    .await?;
    let profile = Profile::parse(&serde_json::to_string(&data["profile"]).map_err(|_| error())?)?;
    let expires = data["expires_at"].as_i64().ok_or_else(error)?;
    let room = Room {
        base,
        code,
        profile,
        expires,
        session: None,
    };
    let view = room.view()?;
    *state.room.lock().await = Some(room);
    *state.last.lock().await = None;
    *state.last_profile.lock().await = None;
    Ok(view)
}
#[tauri::command]
pub async fn room_share(
    consent: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), CommandError> {
    let _gate = state.gate.lock().await;
    if !consent {
        return Err(CommandError {
            message_sl: "Za pošiljanje je potrebna izrecna privolitev.",
        });
    }
    let mut room = state.room.lock().await;
    let r = room.as_mut().ok_or_else(error)?;
    let canonical = r.profile.to_yaml()?;
    if state.last_profile.lock().await.as_deref() != Some(canonical.as_str()) {
        return Err(CommandError {
            message_sl: "Najprej lokalno preveri nespremenjeni profil te sobe.",
        });
    }
    let report = state.last.lock().await.clone().ok_or_else(error)?;
    if r.session.is_none() {
        let data = request(
            &r.base,
            &format!("/rooms/{}/sessions", r.code),
            Method::POST,
            None,
            None,
        )
        .await?;
        r.session = Some(serde_json::from_value(data).map_err(|_| error())?)
    }
    let session = r.session.as_ref().ok_or_else(error)?;
    request(
        &r.base,
        &format!("/rooms/{}/sessions/{}/results", r.code, session.session_id),
        Method::PUT,
        Some(&session.session_token),
        Some(payload(&report)),
    )
    .await?;
    Ok(())
}
#[tauri::command]
pub async fn room_leave(state: tauri::State<'_, AppState>) -> Result<(), CommandError> {
    let _gate = state.gate.lock().await;
    let mut room = state.room.lock().await;
    if let Some(r) = room.as_ref() {
        if let Some(session) = &r.session {
            let path = format!("/rooms/{}/sessions/{}", r.code, session.session_id);
            let response = client()?
                .delete(format!("{}/api/v1{path}", r.base))
                .bearer_auth(&session.session_token)
                .send()
                .await
                .map_err(|_| error())?;
            if !response.status().is_success() && response.status().as_u16() != 404 {
                return Err(error());
            }
        }
    }
    *room = None;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoint_rules() {
        assert!(server_base("http://127.0.0.1:3001").is_ok());
        assert!(server_base("https://school.example").is_ok());
        for s in [
            "http://school.example",
            "https://user:secret@school.example",
            "https://school.example/?token=x",
            "https://school.example/api",
            "file:///secret",
        ] {
            assert!(server_base(s).is_err())
        }
    }
    #[test]
    fn minimized_payload() {
        let report = Report::new(vec![]);
        assert_eq!(
            payload(&report),
            json!({"schema_version":1,"consent":true,"results":[]})
        );
    }
}
