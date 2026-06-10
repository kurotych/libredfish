// Read-only: runs Redfish::get_system against a real BMC and prints
// key fields of the default ComputerSystem. Reads BMC, USER, PASS from env.
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.184 USER=admin PASS=... \
//       cargo run --example get_system

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

    let system = client.get_system().await?;
    println!("id            = {}", system.id);
    println!(
        "manufacturer  = {}",
        system.manufacturer.as_deref().unwrap_or("-")
    );
    println!("model         = {}", system.model.as_deref().unwrap_or("-"));
    println!(
        "serial_number = {}",
        system.serial_number.as_deref().unwrap_or("-")
    );
    println!("sku           = {}", system.sku.as_deref().unwrap_or("-"));
    println!(
        "bios_version  = {}",
        system.bios_version.as_deref().unwrap_or("-")
    );
    println!("power_state   = {:?}", system.power_state);
    println!(
        "asset_tag     = {}",
        system.asset_tag.as_deref().unwrap_or("-")
    );

    Ok(())
}
