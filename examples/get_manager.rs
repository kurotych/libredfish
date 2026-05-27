// Read-only: runs Redfish::get_manager against a real BMC and prints
// key fields of the default Manager. Reads BMC, USER, PASS from env.
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.184 USER=admin PASS=... \
//       cargo run --example get_manager

use std::time::Duration;

use libredfish::reqwest::Url;
use libredfish::{Endpoint, RedfishClientPool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
        )
        .init();

    let url = std::env::var("BMC").expect("set BMC=https://host[:port]");
    let user = std::env::var("USER").ok();
    let pass = std::env::var("PASS").ok();

    let parsed = Url::parse(&url).expect("BMC must be a valid URL");
    let host = parsed
        .host_str()
        .expect("BMC URL must have a host")
        .to_string();
    let port = parsed.port();

    let pool = RedfishClientPool::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()?;

    let client = pool
        .create_client(Endpoint {
            host,
            port,
            user,
            password: pass,
        })
        .await?;

    let manager = client.get_manager().await?;
    println!("id               = {}", manager.id);
    println!("name             = {}", manager.name);
    println!("manager_type     = {}", manager.manager_type);
    println!("model            = {}", manager.model.as_deref().unwrap_or("-"));
    println!("firmware_version = {}", manager.firmware_version);
    println!("uuid             = {}", manager.uuid);
    println!("status.state     = {}", manager.status.state);
    println!(
        "date_time        = {}",
        manager
            .date_time
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| "-".to_string())
    );

    Ok(())
}
