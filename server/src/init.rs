use tracing_subscriber::{filter::LevelFilter, fmt::format::FmtSpan, EnvFilter};
use tokio::net::TcpListener;
use models::config::ServerConfig;
use std::io;

pub async fn connection() -> io::Result<TcpListener> {
    let default_config = ServerConfig::default();
    let config = default_config.init().expect("fatal error occurs at config initialization");
    let addr = format!("{}:{}", config.ip, config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("server running on {}", addr);
    Ok(listener)
}

pub fn trace() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .with_span_events(FmtSpan::FULL)
        .init();
}
