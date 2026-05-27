// WRITE: runs Redfish::boot_first against a real BMC. Changes the
// permanent boot order so the given category boots first.
// Reads BMC, USER, PASS, TARGET from env. TARGET is one of:
//   hdd | harddisk | disk   -> Boot::HardDisk
//   pxe                     -> Boot::Pxe
//   http | uefihttp         -> Boot::UefiHttp
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.183 USER=admin PASS=... TARGET=hdd \
//       cargo run --example boot_first

use std::time::Duration;

use libredfish::reqwest::Url;
use libredfish::{Boot, Endpoint, RedfishClientPool};

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
    let target_raw = std::env::var("TARGET").expect("set TARGET=hdd|pxe|http");

    let target = match target_raw.to_lowercase().as_str() {
        "hdd" | "harddisk" | "disk" => Boot::HardDisk,
        "pxe" => Boot::Pxe,
        "http" | "uefihttp" => Boot::UefiHttp,
        other => anyhow::bail!("TARGET must be hdd|pxe|http, got: {other}"),
    };

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

    println!("setting boot_first target={target}");
    client.boot_first(target).await?;
    println!("done. change applies on next host POST.");

    Ok(())
}
