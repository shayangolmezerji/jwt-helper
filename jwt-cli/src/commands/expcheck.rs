use anyhow::{Result, Context};
use chrono::Utc;
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;


pub fn expcheck_command(token: &str, at_time: Option<&str>) -> Result<()> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid JWT");
    }
    let decoded = general_purpose::URL_SAFE_NO_PAD.decode(parts[1])?;
    let payload: Value = serde_json::from_slice(&decoded)?;
    let exp = payload["exp"].as_i64().context("No exp claim")?;
    let now = if let Some(ts) = at_time {
        ts.parse::<i64>().unwrap_or_else(|_| Utc::now().timestamp())
    } else {
        Utc::now().timestamp()
    };
    if exp < now {
        println!("Token is expired (exp: {}, now: {})", exp, now);
    } else {
        println!("Token is valid (exp: {}, now: {})", exp, now);
    }
    Ok(())
}
