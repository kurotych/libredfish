// Connects to a real BMC and exercises the vendor-detection path so you can
// see which Bmc impl ends up dispatched. Reads BMC, USER, PASS from env.
//
// Run with:
//   RUST_LOG=debug BMC=https://10.213.2.184 USER=admin PASS=... \
//       cargo run --example probe_vendor
//
// Look for these lines in the output:
//   BMC Vendor: AMI                                  (initial detection)
//   BMC Vendor refined: AMI -> GigaComputingAMI      (refinement when applicable)

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

    let endpoint = Endpoint {
        host,
        port,
        user,
        password: pass,
    };

    let client = pool.create_client(endpoint).await?;

    // Touch one endpoint to confirm the dispatched impl actually works.
    let sys = client.get_system().await?;
    println!(
        "ok: system manufacturer={:?} model={:?}",
        sys.manufacturer, sys.model
    );

    Ok(())
}
