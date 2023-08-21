use models::{
    config::{Config, ServerConfig},
    error::{GlobalResult, ExternalError},
};
use time::macros::{offset, format_description};
use tokio::net::TcpListener;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan, time::OffsetTime},
    prelude::__tracing_subscriber_SubscriberExt,
    util::SubscriberInitExt,
};

#[tracing::instrument]
pub async fn listen(ip: &str, port: &str) -> GlobalResult<TcpListener> {
    let addr = format!("{}:{}", ip, port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|_| ExternalError::ListenPort.info(port))?;
    tracing::info!("server running on {}", addr);
    Ok(listener)
}

#[tracing::instrument]
pub fn config() -> GlobalResult<ServerConfig> {
    let config = ServerConfig::init()?;
    Ok(config)
}

/// print log -> std out & files "`exe_dir`/server_log/"
pub fn trace() -> tracing_appender::non_blocking::WorkerGuard {
    let mut log_dir = std::env::current_exe().expect("failed to read cur exe");
    log_dir.pop();
    log_dir.push("server_log");
    let file_appender = tracing_appender::rolling::daily(log_dir, "chat");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let time_offset = offset!(+9);
    let time_description = format_description!("[year]/[month]/[day]-[hour]:[minute]:[second]");
    let timer = OffsetTime::new(time_offset, time_description);

    let file_layer = fmt::layer()
        .with_timer(timer.clone())
        .with_line_number(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::ACTIVE)
        .with_file(true)
        .with_writer(non_blocking)
        .with_ansi(false)
        .compact();

    let std_layer = fmt::layer()
        .with_timer(timer)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_span_events(FmtSpan::ACTIVE)
        .compact();

    tracing_subscriber::registry()
        .with(file_layer)
        .with(std_layer)
        .init();

    guard
}
