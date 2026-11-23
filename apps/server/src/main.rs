#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = axum::Router::new().route("/health", axum::routing::get(|| async { "ok" }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;
    println!("Preflight API: http://127.0.0.1:3001/health");
    axum::serve(listener, app).await
}
