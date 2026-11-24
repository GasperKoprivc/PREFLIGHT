use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    Router,
};
use preflight_server::{cleanup, connect, now, router, state, AppState};
use serde_json::{json, Value};
use tower::ServiceExt;
const PROFILE: &str = include_str!("../../../crates/preflight-core/tests/fixtures/runtime.yaml");
async fn setup() -> (Router, AppState) {
    let s = state(connect("sqlite::memory:").await.unwrap(), 86400);
    (router(s.clone(), None).unwrap(), s)
}
async fn call(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(path);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"))
    }
    if body.is_some() {
        req = req.header("Content-Type", "application/json")
    };
    let r = app
        .clone()
        .oneshot(
            req.body(
                body.map(|v| Body::from(v.to_string()))
                    .unwrap_or_else(Body::empty),
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let bytes = to_bytes(r.into_body(), 200000).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
async fn create(app: &Router) -> Value {
    let (s, v) = call(
        app,
        "POST",
        "/api/v1/rooms",
        None,
        Some(json!({"profile_yaml":PROFILE})),
    )
    .await;
    assert_eq!(s, 201);
    v
}
async fn join(app: &Router, code: &str) -> Value {
    let (s, v) = call(
        app,
        "POST",
        &format!("/api/v1/rooms/{code}/sessions"),
        None,
        None,
    )
    .await;
    assert_eq!(s, 201);
    v
}
fn report() -> Value {
    json!({"schema_version":1,"consent":true,"results":[{"check_id":"python","status":"pass","installed_version":"3.12.0","reason_code":"satisfied"},{"check_id":"git","status":"pass","installed_version":"2.53.0"},{"check_id":"node","status":"warning","installed_version":"24.0.0","reason_code":"version_mismatch"}]})
}
#[tokio::test]
async fn room_session_lifecycle_and_authorization() {
    let (app, s) = setup().await;
    let r = create(&app).await;
    let code = r["room_code"].as_str().unwrap();
    let teacher = r["teacher_token"].as_str().unwrap();
    assert_eq!(code.len(), 8);
    assert_eq!(teacher.len(), 64);
    let p = format!("/api/v1/rooms/{code}");
    assert_eq!(
        call(&app, "GET", &format!("{p}/profile"), None, None)
            .await
            .0,
        200
    );
    let a = join(&app, code).await;
    let b = join(&app, code).await;
    let id = a["session_id"].as_str().unwrap();
    let t = a["session_token"].as_str().unwrap();
    let bt = b["session_token"].as_str().unwrap();
    let upload = format!("{p}/sessions/{id}/results");
    assert_eq!(
        call(&app, "GET", &format!("{p}/results"), None, None)
            .await
            .0,
        401
    );
    assert_eq!(
        call(&app, "GET", &format!("{p}/results"), Some(t), None)
            .await
            .0,
        401
    );
    assert_eq!(
        call(&app, "PUT", &upload, Some(bt), Some(report())).await.0,
        401
    );
    assert_eq!(
        call(&app, "PUT", &upload, Some(teacher), Some(report()))
            .await
            .0,
        401
    );
    assert_eq!(
        call(&app, "PUT", &upload, Some(t), Some(report())).await.0,
        204
    );
    let (status, v) = call(&app, "GET", &format!("{p}/results"), Some(teacher), None).await;
    assert_eq!(status, 200);
    assert_eq!(v["aggregate"]["pass"], 2);
    assert_eq!(v["sessions"].as_array().unwrap().len(), 2);
    assert!(!v.to_string().contains(t));
    assert!(!v.to_string().contains(teacher));
    let stored: String = sqlx::query_scalar("SELECT teacher_hash FROM rooms WHERE code=?")
        .bind(code)
        .fetch_one(&s.pool)
        .await
        .unwrap();
    assert_ne!(stored, teacher);
    assert_eq!(
        call(
            &app,
            "DELETE",
            &format!("{p}/sessions/{id}"),
            Some(bt),
            None
        )
        .await
        .0,
        401
    );
    assert_eq!(
        call(&app, "DELETE", &format!("{p}/sessions/{id}"), Some(t), None)
            .await
            .0,
        204
    );
    assert_eq!(
        call(&app, "PUT", &upload, Some(t), Some(report())).await.0,
        401
    );
    assert_eq!(call(&app, "DELETE", &p, Some(bt), None).await.0, 401);
    assert_eq!(call(&app, "DELETE", &p, Some(teacher), None).await.0, 204);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(&s.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    assert_eq!(
        call(&app, "GET", &format!("{p}/profile"), None, None)
            .await
            .0,
        404
    );
}
#[tokio::test]
async fn expiry_removes_everything() {
    let (app, s) = setup().await;
    let r = create(&app).await;
    let code = r["room_code"].as_str().unwrap();
    join(&app, code).await;
    sqlx::query("UPDATE rooms SET expires_at=?")
        .bind(now() - 1)
        .execute(&s.pool)
        .await
        .unwrap();
    assert_eq!(cleanup(&s).await.unwrap(), 1);
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("/api/v1/rooms/{code}/results"),
            r["teacher_token"].as_str(),
            None
        )
        .await
        .0,
        404
    );
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(&s.pool)
        .await
        .unwrap();
    assert_eq!(count, 0)
}
#[tokio::test]
async fn validation_and_consent() {
    let (app, _) = setup().await;
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/rooms",
            None,
            Some(json!({"profile_yaml":"invalid"}))
        )
        .await
        .0,
        400
    );
    let r = create(&app).await;
    let code = r["room_code"].as_str().unwrap();
    let a = join(&app, code).await;
    let path = format!(
        "/api/v1/rooms/{code}/sessions/{}/results",
        a["session_id"].as_str().unwrap()
    );
    let t = a["session_token"].as_str();
    for field in ["consent", "schema_version", "results"] {
        let mut payload = report();
        payload[field] = match field {
            "consent" => json!(false),
            "schema_version" => json!(2),
            _ => json!([]),
        };
        assert_eq!(call(&app, "PUT", &path, t, Some(payload)).await.0, 400)
    }
    for key in ["check_id", "installed_version", "reason_code"] {
        let mut payload = report();
        payload["results"][0][key] = json!("C:/Users/private");
        assert!(call(&app, "PUT", &path, t, Some(payload))
            .await
            .0
            .is_client_error())
    }
    let mut payload = report();
    payload["results"][0]["stdout"] = json!("secret");
    assert!(call(&app, "PUT", &path, t, Some(payload))
        .await
        .0
        .is_client_error());
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("/api/v1/rooms/{code}/results?teacher_token=secret"),
            None,
            None
        )
        .await
        .0,
        400
    )
}
#[tokio::test]
async fn cross_room_session_is_rejected() {
    let (app, _) = setup().await;
    let a = create(&app).await;
    let b = create(&app).await;
    let session = join(&app, a["room_code"].as_str().unwrap()).await;
    let path = format!(
        "/api/v1/rooms/{}/sessions/{}/results",
        b["room_code"].as_str().unwrap(),
        session["session_id"].as_str().unwrap()
    );
    assert_eq!(
        call(
            &app,
            "PUT",
            &path,
            session["session_token"].as_str(),
            Some(report())
        )
        .await
        .0,
        401
    )
}
#[tokio::test]
async fn body_limit_and_rate_limit() {
    let (app, _) = setup().await;
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/rooms",
        None,
        Some(json!({"profile_yaml":"x".repeat(100000)})),
    )
    .await;
    assert_eq!(status, 413);
    for _ in 0..599 {
        call(&app, "GET", "/health", None, None).await;
    }
    assert_eq!(call(&app, "GET", "/health", None, None).await.0, 429)
}
#[tokio::test]
async fn cors_and_no_store() {
    let (_, s) = setup().await;
    let app = router(s, Some("https://teacher.example")).unwrap();
    let r = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("Origin", "https://evil.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(
        r.headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap()),
        Some("https://evil.example")
    );
    assert_eq!(r.headers()["cache-control"], "no-store")
}

#[tokio::test]
async fn expired_access_triggers_cleanup_without_background_job() {
    let (app, s) = setup().await;
    let r = create(&app).await;
    let code = r["room_code"].as_str().unwrap();
    let a = join(&app, code).await;
    sqlx::query("UPDATE rooms SET expires_at=? WHERE code=?")
        .bind(now() - 1)
        .bind(code)
        .execute(&s.pool)
        .await
        .unwrap();
    let p = format!(
        "/api/v1/rooms/{code}/sessions/{}/results",
        a["session_id"].as_str().unwrap()
    );
    assert_eq!(
        call(&app, "PUT", &p, a["session_token"].as_str(), Some(report()))
            .await
            .0,
        404
    );
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(&s.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
