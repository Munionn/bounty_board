use server::{run, AppConfig};

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();
    run(config).await.expect("server failed to start");
}
