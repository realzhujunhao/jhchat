use tracing_subscriber::{filter::LevelFilter, fmt::format::FmtSpan, EnvFilter};
use tokio::net::{TcpListener, TcpStream};
use models::config::ServerConfig;
use std::io;

pub async fn connection() -> io::Result<TcpListener> {
    let default_config = ServerConfig::default();
    let config = default_config.init().expect("fatal error occurs at config initialization");
    let addr = format!("{}:{}", config.ip, config.port);
    Ok(TcpListener::bind(&addr).await?)
}

pub fn trace() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .with_span_events(FmtSpan::FULL)
        .init();
}
