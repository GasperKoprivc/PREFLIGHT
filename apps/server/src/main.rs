use preflight_server::{cleanup, connect, router, state};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://preflight.sqlite".into());
    let bind = std::env::var("PREFLIGHT_BIND").unwrap_or_else(|_| "127.0.0.1:3001".into());
    let ttl = std::env::var("PREFLIGHT_ROOM_TTL_SECONDS")
        .unwrap_or_else(|_| "86400".into())
        .parse::<i64>()?;
    if !(60..=86400).contains(&ttl) {
        return Err("PREFLIGHT_ROOM_TTL_SECONDS must be between 60 and 86400".into());
    }
    let pool = connect(&url).await?;
    let state = state(pool, ttl);
    cleanup(&state).await?;
    let origin = std::env::var("PREFLIGHT_ALLOWED_ORIGIN").ok();
    let app = router(state.clone(), origin.as_deref()).map_err(|_| "Invalid CORS origin")?;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if cleanup(&state).await.is_err() {
                eprintln!("Čiščenje poteklih sob ni uspelo.")
            }
        }
    });
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("Preflight API je pripravljen.");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
