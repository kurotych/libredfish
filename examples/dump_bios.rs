// Dumps the BIOS attributes from a live BMC so we can map intent ->
// vendor-specific attribute names. Reads BMC, USER, PASS from env.
//
// Run with:
//   BMC=https://10.213.2.184 USER=admin PASS=... \
//       cargo run --example dump_bios

use std::time::Duration;

use libredfish::reqwest::Url;
use libredfish::{Endpoint, RedfishClientPool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let bios = client.bios().await?;
    let attrs = bios
        .get("Attributes")
        .and_then(|v| v.as_object())
        .expect("BIOS body should contain an Attributes object");

    let mut entries: Vec<(&String, &serde_json::Value)> = attrs.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    println!("{} BIOS attributes:", entries.len());
    for (k, v) in entries {
        println!("{k} = {v}");
    }

    Ok(())
}
