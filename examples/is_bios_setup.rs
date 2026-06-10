// Read-only check: runs Redfish::is_bios_setup against a real BMC and prints
// the boolean result. Reads BMC, USER, PASS from env. Optional BOOT_MAC is
// forwarded (most vendor impls ignore it, but some use it for boot-order
// validation).
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.184 USER=admin PASS=... \
//       cargo run --example is_bios_setup
//
//   BOOT_MAC=B8:E9:24:17:6D:72 cargo run --example is_bios_setup

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
    let boot_mac = std::env::var("BOOT_MAC").ok();

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

    let ok = client.is_bios_setup(boot_mac.as_deref()).await?;
    println!("is_bios_setup = {}", ok);

    Ok(())
}
