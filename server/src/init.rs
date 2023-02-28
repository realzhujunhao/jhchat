use models::{
    config::ServerConfig,
    error::{Error, Result},
};
use tokio::net::TcpListener;
use tracing_subscriber::{filter::LevelFilter, fmt::format::FmtSpan, EnvFilter};
use std::fs::create_dir_all;

pub async fn connection(ip: &str, port: &str) -> Result<TcpListener> {
    let addr = format!("{}:{}", ip, port);
    let listener = TcpListener::bind(&addr).await.map_err(|_| Error::Listen(port.into()))?;
    tracing::info!("server running on {}", addr);
    Ok(listener)
}

pub fn config() -> Result<ServerConfig> {
    let default_config = ServerConfig::default();
    let config = default_config.init().map_err(|_| Error::Config)?;
    Ok(config)
}

pub fn file_structure(file_dir: &str) {
    let _ = create_dir_all(file_dir);
}

pub fn trace() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .with_span_events(FmtSpan::FULL)
        .init();
}
