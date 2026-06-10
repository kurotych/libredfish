// Read-only: runs Redfish::get_service_root against a real BMC and prints
// key fields of the ServiceRoot. Reads BMC, USER, PASS from env.
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.184 USER=admin PASS=... \
//       cargo run --example get_service_root

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

    let sr = client.get_service_root().await?;
    println!("redfish_version = {}", sr.redfish_version);
    println!("product         = {}", sr.product.as_deref().unwrap_or("-"));
    println!("vendor          = {}", sr.vendor.as_deref().unwrap_or("-"));
    println!(
        "vendor_string   = {}",
        sr.vendor_string().as_deref().unwrap_or("-")
    );
    println!("uuid            = {}", sr.uuid.as_deref().unwrap_or("-"));
    println!(
        "systems         = {}",
        sr.systems
            .as_ref()
            .map(|o| o.odata_id.as_str())
            .unwrap_or("-")
    );
    println!(
        "managers        = {}",
        sr.managers
            .as_ref()
            .map(|o| o.odata_id.as_str())
            .unwrap_or("-")
    );
    println!(
        "chassis         = {}",
        sr.chassis
            .as_ref()
            .map(|o| o.odata_id.as_str())
            .unwrap_or("-")
    );
    println!(
        "account_service = {}",
        sr.account_service
            .as_ref()
            .map(|o| o.odata_id.as_str())
            .unwrap_or("-")
    );
    println!(
        "tasks           = {}",
        sr.tasks
            .as_ref()
            .map(|o| o.odata_id.as_str())
            .unwrap_or("-")
    );

    Ok(())
}
