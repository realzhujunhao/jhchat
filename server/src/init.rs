use models::{
    config::ServerConfig,
    error::{Error, Result},
};
use tokio::net::TcpListener;
use tracing_subscriber::{filter::LevelFilter, fmt::format::FmtSpan, EnvFilter};

pub async fn connection() -> Result<TcpListener> {
    let default_config = ServerConfig::default();
    let config = default_config.init().map_err(|_| Error::Config)?;
    let addr = format!("{}:{}", config.ip, config.port);
    let listener = TcpListener::bind(&addr).await.map_err(|_| Error::Listen)?;
    tracing::info!("server running on {}", addr);
    Ok(listener)
}

pub fn trace() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .with_span_events(FmtSpan::FULL)
        .init();
}
