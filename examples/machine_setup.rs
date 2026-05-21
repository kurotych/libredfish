// Runs Redfish::machine_setup against a real BMC, then prints the resulting
// machine_setup_status diffs. Reads BMC, USER, PASS from env. Optional BOOT_MAC
// is forwarded to machine_setup_status so the boot-order check is meaningful.
//
// WARNING: this PATCHes BIOS attributes (serial console, TPM Clear, SR-IOV,
// network stack, etc.) on the target system. Only run against a BMC you own.
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.184 USER=admin PASS=... \
//       cargo run --example machine_setup
//
//   # Optional: also pass the boot interface MAC so machine_setup_status
//   # can evaluate the boot-order diff.
//   BOOT_MAC=B8:E9:24:17:6D:72 cargo run --example machine_setup

use std::collections::HashMap;
use std::time::Duration;

use libredfish::reqwest::Url;
use libredfish::{BiosProfileType, BiosProfileVendor, Endpoint, RedfishClientPool};

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
        .timeout(Duration::from_secs(60))
        .build()?;

    let client = pool
        .create_client(Endpoint {
            host,
            port,
            user,
            password: pass,
        })
        .await?;

    let bios_profiles: BiosProfileVendor = HashMap::new();
    let oem_manager_profiles: BiosProfileVendor = HashMap::new();

    println!("Calling machine_setup...");
    let job_id = client
        .machine_setup(
            boot_mac.as_deref(),
            &bios_profiles,
            BiosProfileType::default(),
            &oem_manager_profiles,
        )
        .await?;
    println!("machine_setup returned job_id={:?}", job_id);

    println!("Checking machine_setup_status...");
    let status = client.machine_setup_status(boot_mac.as_deref()).await?;
    println!("is_done={}", status.is_done);
    if status.diffs.is_empty() {
        println!("no diffs");
    } else {
        for d in &status.diffs {
            println!("  {}: expected={} actual={}", d.key, d.expected, d.actual);
        }
    }

    Ok(())
}
