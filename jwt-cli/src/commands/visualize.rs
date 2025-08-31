use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;


pub fn visualize_command(token: &str, mode: &str) -> Result<()> {
    match mode {
        "json" => {
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() != 3 {
                anyhow::bail!("Invalid JWT");
            }
            let decode = |b64| {
                let decoded = general_purpose::URL_SAFE_NO_PAD.decode(b64)?;
                Ok::<Value, anyhow::Error>(serde_json::from_slice(&decoded)?)
            };
            let header = decode(parts[0])?;
            let payload = decode(parts[1])?;
            println!("Header:");
            serde_json::to_writer_pretty(std::io::stdout(), &header)?;
            println!("\nPayload:");
            serde_json::to_writer_pretty(std::io::stdout(), &payload)?;
        }
        "qr" => {

            println!("[QR] Visualization not implemented in this stub");
        }
        _ => anyhow::bail!("Unknown visualization mode: {}", mode),
    }
    Ok(())
}
