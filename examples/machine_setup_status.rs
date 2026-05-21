// Read-only: runs Redfish::machine_setup_status against a real BMC and prints
// the diffs so you can see exactly which attributes are off. Reads BMC, USER,
// PASS from env. Optional BOOT_MAC enables the boot-order diff.
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.184 USER=admin PASS=... \
//       cargo run --example machine_setup_status

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

    let status = client.machine_setup_status(boot_mac.as_deref()).await?;
    println!("is_done = {}", status.is_done);
    if status.diffs.is_empty() {
        println!("no diffs");
    } else {
        println!("{} diff(s):", status.diffs.len());
        for d in &status.diffs {
            println!("  {}: expected={} actual={}", d.key, d.expected, d.actual);
        }
    }

    Ok(())
}
