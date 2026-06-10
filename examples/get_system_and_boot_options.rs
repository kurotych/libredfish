// Read-only: fetches the default ComputerSystem and the expanded BootOptions
// collection (mirroring the internal `get_system_and_boot_options` helper).
// Two HTTP calls: GET /Systems/{id} and GET <Boot.BootOptions@odata.id>?$expand=.($levels=1).
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.184 USER=admin PASS=... \
//       cargo run --example get_system_and_boot_options

use std::time::Duration;

use libredfish::model::BootOption;
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
    let boot_options_id = system
        .boot
        .boot_options
        .clone()
        .ok_or_else(|| anyhow::anyhow!("system.Boot has no BootOptions link"))?;
    let all_boot_options = client
        .get_collection(boot_options_id)
        .await?
        .try_get::<BootOption>()?
        .members;

    println!("system id   = {}", system.id);
    println!("boot order  = {:?}", system.boot.boot_order);
    println!("found {} boot option(s):", all_boot_options.len());
    for opt in &all_boot_options {
        println!(
            "- {:<10} ref={:<10} enabled={:?} alias={:?}  display={}",
            opt.id, opt.boot_option_reference, opt.boot_option_enabled, opt.alias, opt.display_name,
        );
    }

    Ok(())
}
