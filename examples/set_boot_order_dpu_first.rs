// WRITE: runs Redfish::set_boot_order_dpu_first against a real BMC.
// Finds the boot option matching MAC (platform-specific: GigaComputing AMI
// looks for UEFI HTTP IPv4 entries containing the MAC) and moves it to
// the front of BootOrder. Reads BMC, USER, PASS, MAC from env.
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.184 USER=admin PASS=... \
//       MAC=58:A2:E1:54:6F:8A cargo run --example set_boot_order_dpu_first

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
    let mac = std::env::var("MAC").expect("set MAC=aa:bb:cc:dd:ee:ff");

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

    println!("setting boot order DPU-first for MAC {mac}");
    match client.set_boot_order_dpu_first(&mac).await? {
        Some(job) => println!("submitted; job/task = {job}"),
        None => println!("applied (no job returned)"),
    }

    Ok(())
}
