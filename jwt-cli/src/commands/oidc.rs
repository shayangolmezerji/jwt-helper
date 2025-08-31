
use anyhow::{Result, Context};
use reqwest::blocking::Client;
use serde_json::Value;


pub fn oidc_command(endpoint: &str, token: &str) -> Result<()> {
    let client = Client::new();
    let resp = client.post(endpoint)
        .form(&[ ("token", token) ])
        .send()
        .with_context(|| format!("Failed to introspect token at {}", endpoint))?;
    let json: Value = resp.json().with_context(|| "Failed to parse introspection response")?;
    println!("Introspection result:");
    serde_json::to_writer_pretty(std::io::stdout(), &json)?;
    println!();
    Ok(())
}
