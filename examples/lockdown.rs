// WRITE: runs Redfish::lockdown against a real BMC. Toggles the host
// interface (and on other platforms, KCS/USB BIOS attrs). Reads BMC, USER,
// PASS, TARGET from env. TARGET is "enabled" (lock) or "disabled" (unlock).
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.184 USER=admin PASS=... TARGET=enabled \
//       cargo run --example lockdown

use std::time::Duration;

use libredfish::reqwest::Url;
use libredfish::{EnabledDisabled, Endpoint, RedfishClientPool};

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
    let target_raw = std::env::var("TARGET").expect("set TARGET=enabled|disabled");

    let target = match target_raw.to_lowercase().as_str() {
        "enabled" | "enable" | "lock" => EnabledDisabled::Enabled,
        "disabled" | "disable" | "unlock" => EnabledDisabled::Disabled,
        other => anyhow::bail!("TARGET must be enabled|disabled, got: {other}"),
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

    println!("applying lockdown target={target:?}");
    client.lockdown(target).await?;
    println!("done. re-reading status…");

    let status = client.lockdown_status().await?;
    let label = if status.is_fully_enabled() {
        "Enabled (locked)"
    } else if status.is_fully_disabled() {
        "Disabled (unlocked)"
    } else {
        "Partial"
    };
    println!("status  = {label}");
    println!("message = {}", status.message());

    Ok(())
}
