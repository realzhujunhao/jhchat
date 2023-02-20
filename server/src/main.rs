use models::config::ServerConfig;

#[tokio::main]
async fn main() {
    let default_config = ServerConfig::default();
    let config = default_config.init().unwrap();
    println!("{:?}", config);
}
