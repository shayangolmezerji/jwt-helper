use anyhow::Result;
use serde_json::Value;
use base64::{engine::general_purpose, Engine as _};


pub fn diff_command(token1: &str, token2: &str) -> Result<()> {
    let parts1: Vec<&str> = token1.split('.').collect();
    let parts2: Vec<&str> = token2.split('.').collect();
    if parts1.len() != 3 || parts2.len() != 3 {
        anyhow::bail!("Both tokens must have 3 parts");
    }
    let decode = |b64| {
        let decoded = general_purpose::URL_SAFE_NO_PAD.decode(b64)?;
        Ok::<Value, anyhow::Error>(serde_json::from_slice(&decoded)?)
    };
    let header1 = decode(parts1[0])?;
    let header2 = decode(parts2[0])?;
    let payload1 = decode(parts1[1])?;
    let payload2 = decode(parts2[1])?;
    println!("Header diff: {}", if header1 == header2 { "identical" } else { "different" });
    println!("Payload diff: {}", if payload1 == payload2 { "identical" } else { "different" });
    Ok(())
}
