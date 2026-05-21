// Dumps the pending BIOS attributes (the /Bios/SD bucket on AMI, /Bios/Settings
// on most others) so you can see what's queued for the next BIOS POST.
// Reads BMC, USER, PASS from env.
//
// Run with:
//   BMC=https://10.213.2.184 USER=admin PASS=... \
//       cargo run --example dump_pending

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

    let pending = client.pending().await?;
    let mut entries: Vec<(&String, &serde_json::Value)> = pending.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    println!("{} pending BIOS attribute(s):", entries.len());
    for (k, v) in entries {
        println!("{k} = {v}");
    }

    Ok(())
}
