use axum::{
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use preflight_core::Profile;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use subtle::ConstantTimeEq;
use tower_http::{cors::CorsLayer, set_header::SetResponseHeaderLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub ttl_seconds: i64,
    limiter: Arc<Mutex<(Instant, u32)>>,
}
#[derive(Debug)]
pub struct ApiError(StatusCode, &'static str);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.0,
            Json(serde_json::json!({"error":{"message_sl":self.1}})),
        )
            .into_response()
    }
}
impl From<sqlx::Error> for ApiError {
    fn from(_: sqlx::Error) -> Self {
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Podatkovna storitev ni na voljo.",
        )
    }
}
type ApiResult<T> = Result<T, ApiError>;
fn bad() -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, "Vhodni podatki niso veljavni.")
}
fn denied() -> ApiError {
    ApiError(StatusCode::UNAUTHORIZED, "Dostop ni dovoljen.")
}
fn missing() -> ApiError {
    ApiError(
        StatusCode::NOT_FOUND,
        "Soba ali seja ni na voljo oziroma je potekla.",
    )
}
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
fn token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
fn hash(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
fn auth(headers: &HeaderMap, expected: &str) -> ApiResult<()> {
    let t = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(denied)?;
    if bool::from(hash(t).as_bytes().ct_eq(expected.as_bytes())) {
        Ok(())
    } else {
        Err(denied())
    }
}
fn valid_code(code: &str) -> bool {
    code.len() == 8
        && code
            .bytes()
            .all(|b| b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789".contains(&b))
}
fn valid_session(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok() && id.len() == 36
}
pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePoolOptions::new()
        .max_connections(if url.contains(":memory:") { 1 } else { 5 })
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
pub fn state(pool: SqlitePool, ttl_seconds: i64) -> AppState {
    AppState {
        pool,
        ttl_seconds,
        limiter: Arc::new(Mutex::new((Instant::now(), 0))),
    }
}
pub async fn cleanup(state: &AppState) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM rooms WHERE expires_at <= ?")
        .bind(now())
        .execute(&state.pool)
        .await?
        .rows_affected())
}
async fn guard(State(s): State<AppState>, request: Request, next: Next) -> Response {
    if !request.uri().query().unwrap_or("").is_empty() {
        return bad().into_response();
    }
    let allowed = match s.limiter.lock() {
        Ok(mut l) => {
            if l.0.elapsed() >= Duration::from_secs(60) {
                *l = (Instant::now(), 0)
            }
            l.1 += 1;
            l.1 <= 600
        }
        Err(_) => false,
    };
    if !allowed {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, "60")],
            Json(serde_json::json!({"error":{"message_sl":"Preveč zahtev. Poskusi čez minuto."}})),
        )
            .into_response();
    }
    next.run(request).await
}
pub fn router(s: AppState, allowed_origin: Option<&str>) -> Result<Router, ApiError> {
    let mut cors = CorsLayer::new();
    if let Some(origin) = allowed_origin {
        let parsed = url_origin(origin)?;
        if parsed != origin {
            return Err(bad());
        }
        let value = origin
            .parse::<axum::http::HeaderValue>()
            .map_err(|_| bad())?;
        cors = cors
            .allow_origin(value)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);
    }
    Ok(Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/v1/rooms", post(create_room))
        .route("/api/v1/rooms/{code}/profile", get(room_profile))
        .route("/api/v1/rooms/{code}/sessions", post(join_room))
        .route(
            "/api/v1/rooms/{code}/sessions/{id}/results",
            put(update_results),
        )
        .route("/api/v1/rooms/{code}/sessions/{id}", delete(delete_session))
        .route("/api/v1/rooms/{code}/results", get(results))
        .route("/api/v1/rooms/{code}", delete(delete_room))
        .layer(DefaultBodyLimit::max(96 * 1024))
        .layer(middleware::from_fn_with_state(s.clone(), guard))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        ))
        .layer(cors)
        .with_state(s))
}
async fn room(s: &AppState, code: &str) -> ApiResult<sqlx::sqlite::SqliteRow> {
    if !valid_code(code) {
        return Err(bad());
    }
    cleanup(s).await?;
    sqlx::query("SELECT * FROM rooms WHERE code = ? AND expires_at > ?")
        .bind(code)
        .bind(now())
        .fetch_optional(&s.pool)
        .await?
        .ok_or_else(missing)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateRoom {
    profile_yaml: String,
}
async fn create_room(
    State(s): State<AppState>,
    Json(body): Json<CreateRoom>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let profile = Profile::parse(&body.profile_yaml).map_err(|_| bad())?;
    let yaml = profile.to_yaml().map_err(|_| bad())?;
    cleanup(&s).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rooms")
        .fetch_one(&s.pool)
        .await?;
    if count >= 1000 {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Dosežena je omejitev sob.",
        ));
    }
    let secret = token();
    let expiry = now() + s.ttl_seconds;
    for _ in 0..5 {
        let random = uuid::Uuid::new_v4();
        let alphabet = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let code: String = random.as_bytes()[..8]
            .iter()
            .map(|b| alphabet[(b % 32) as usize] as char)
            .collect();
        let inserted=sqlx::query("INSERT OR IGNORE INTO rooms(code,teacher_hash,profile_yaml,created_at,expires_at) VALUES(?,?,?,?,?)").bind(&code).bind(hash(&secret)).bind(&yaml).bind(now()).bind(expiry).execute(&s.pool).await?;
        if inserted.rows_affected() == 1 {
            return Ok((
                StatusCode::CREATED,
                Json(
                    serde_json::json!({"room_code":code,"teacher_token":secret,"expires_at":expiry}),
                ),
            ));
        }
    }
    Err(ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "Sobe trenutno ni mogoče ustvariti.",
    ))
}
async fn room_profile(
    State(s): State<AppState>,
    Path(code): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let r = room(&s, &code).await?;
    let p = Profile::parse(r.get("profile_yaml")).map_err(|_| missing())?;
    Ok(Json(
        serde_json::json!({"profile":p,"expires_at":r.get::<i64,_>("expires_at")}),
    ))
}
async fn join_room(
    State(s): State<AppState>,
    Path(code): Path<String>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let r = room(&s, &code).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let secret = token();
    let alias = format!(
        "Udeleženec-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    );
    let inserted=sqlx::query("INSERT INTO sessions(id,room_code,token_hash,alias,updated_at) SELECT ?,?,?,?,? WHERE EXISTS(SELECT 1 FROM rooms WHERE code=? AND expires_at>?) AND (SELECT COUNT(*) FROM sessions WHERE room_code=?)<200").bind(&id).bind(&code).bind(hash(&secret)).bind(&alias).bind(now()).bind(&code).bind(now()).bind(&code).execute(&s.pool).await?;
    if inserted.rows_affected() != 1 {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "Soba ni na voljo ali je polna.",
        ));
    }
    Ok((
        StatusCode::CREATED,
        Json(
            serde_json::json!({"session_id":id,"session_token":secret,"alias":alias,"expires_at":r.get::<i64,_>("expires_at")}),
        ),
    ))
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SharedResult {
    check_id: String,
    status: SharedStatus,
    installed_version: Option<String>,
    reason_code: Option<SharedReason>,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum SharedStatus {
    Pass,
    Fail,
    Warning,
    Unknown,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum SharedReason {
    Satisfied,
    VersionMismatch,
    NotFound,
    Timeout,
    OutputLimit,
    ProcessFailed,
    UnrecognizedVersion,
    JdkMissing,
    JdkMismatch,
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
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Update {
    schema_version: u32,
    consent: bool,
    results: Vec<SharedResult>,
}
async fn session(s: &AppState, code: &str, id: &str, headers: &HeaderMap) -> ApiResult<()> {
    if !valid_session(id) {
        return Err(bad());
    }
    let row = sqlx::query("SELECT token_hash FROM sessions WHERE id=? AND room_code=?")
        .bind(id)
        .bind(code)
        .fetch_optional(&s.pool)
        .await?
        .ok_or_else(denied)?;
    auth(headers, row.get("token_hash"))
}
async fn update_results(
    State(s): State<AppState>,
    Path((code, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<Update>,
) -> ApiResult<StatusCode> {
    let r = room(&s, &code).await?;
    session(&s, &code, &id, &headers).await?;
    let p = Profile::parse(r.get("profile_yaml")).map_err(|_| missing())?;
    if body.schema_version != 1 || !body.consent || body.results.len() != p.checks().len() {
        return Err(bad());
    }
    let mut seen = std::collections::HashSet::new();
    for result in &body.results {
        if !p.checks().iter().any(|c| c.id == result.check_id) || !seen.insert(&result.check_id) {
            return Err(bad());
        }
        if let Some(v) = &result.installed_version {
            let parsed = semver::Version::parse(v).map_err(|_| bad())?;
            if v.len() > 40 || !parsed.pre.is_empty() || !parsed.build.is_empty() {
                return Err(bad());
            }
        }
    }
    let json = serde_json::to_string(&body.results).map_err(|_| bad())?;
    let changed=sqlx::query("UPDATE sessions SET results_json=?,updated_at=? WHERE id=? AND room_code=? AND EXISTS(SELECT 1 FROM rooms WHERE code=? AND expires_at>?)").bind(json).bind(now()).bind(id).bind(&code).bind(&code).bind(now()).execute(&s.pool).await?;
    if changed.rows_affected() != 1 {
        return Err(missing());
    }
    Ok(StatusCode::NO_CONTENT)
}
async fn delete_session(
    State(s): State<AppState>,
    Path((code, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    room(&s, &code).await?;
    session(&s, &code, &id, &headers).await?;
    sqlx::query("DELETE FROM sessions WHERE id=? AND room_code=?")
        .bind(id)
        .bind(code)
        .execute(&s.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn results(
    State(s): State<AppState>,
    Path(code): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    let r = room(&s, &code).await?;
    auth(&headers, r.get("teacher_hash"))?;
    let rows =
        sqlx::query("SELECT id,alias,results_json FROM sessions WHERE room_code=? ORDER BY id")
            .bind(code)
            .fetch_all(&s.pool)
            .await?;
    let mut sessions = Vec::new();
    let mut totals = serde_json::json!({"pass":0,"fail":0,"warning":0,"unknown":0});
    for row in rows {
        let json: Option<String> = row.get("results_json");
        let shared: Option<Vec<SharedResult>> = json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|_| {
                ApiError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Rezultatov ni mogoče prebrati.",
                )
            })?;
        if let Some(items) = &shared {
            for item in items {
                let key = match item.status {
                    SharedStatus::Pass => "pass",
                    SharedStatus::Fail => "fail",
                    SharedStatus::Warning => "warning",
                    SharedStatus::Unknown => "unknown",
                };
                totals[key] = serde_json::json!(totals[key].as_u64().unwrap_or(0) + 1)
            }
        }
        sessions.push(serde_json::json!({"session_id":row.get::<String,_>("id"),"alias":row.get::<String,_>("alias"),"results":shared}));
    }
    Ok(Json(
        serde_json::json!({"schema_version":1,"sessions":sessions,"aggregate":totals,"expires_at":r.get::<i64,_>("expires_at")}),
    ))
}
async fn delete_room(
    State(s): State<AppState>,
    Path(code): Path<String>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    let r = room(&s, &code).await?;
    auth(&headers, r.get("teacher_hash"))?;
    sqlx::query("DELETE FROM rooms WHERE code=?")
        .bind(code)
        .execute(&s.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

fn url_origin(input: &str) -> ApiResult<String> {
    let u = reqwest_url(input)?;
    Ok(u)
}
fn reqwest_url(input: &str) -> ApiResult<String> {
    let u = url::Url::parse(input).map_err(|_| bad())?;
    if !["http", "https"].contains(&u.scheme())
        || !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
    {
        return Err(bad());
    }
    Ok(u.origin().ascii_serialization())
}
