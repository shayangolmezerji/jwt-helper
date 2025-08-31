use anyhow::Result;

pub fn decode_jwt(token: &str) -> Result<()> {
    use anyhow::{bail, Context};
    use base64::{engine::general_purpose, Engine as _};
    use serde_json::Value;
    use std::io;

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        bail!("Invalid JWT: must have 3 parts (header.payload.signature)");
    }
    let header = {
        let decoded = general_purpose::URL_SAFE_NO_PAD.decode(parts[0])
            .with_context(|| "Failed to base64-decode header")?;
        serde_json::from_slice::<Value>(&decoded)
            .with_context(|| "Failed to parse header as JSON")?
    };
    let payload = {
        let decoded = general_purpose::URL_SAFE_NO_PAD.decode(parts[1])
            .with_context(|| "Failed to base64-decode payload")?;
        serde_json::from_slice::<Value>(&decoded)
            .with_context(|| "Failed to parse payload as JSON")?
    };
    let signature = {
        let decoded = general_purpose::URL_SAFE_NO_PAD.decode(parts[2])
            .with_context(|| "Failed to base64-decode signature")?;
        hex::encode(decoded)
    };

    println!("Header:");
    serde_json::to_writer_pretty(io::stdout(), &header)?;
    println!("\nPayload:");
    serde_json::to_writer_pretty(io::stdout(), &payload)?;
    println!("\nSignature (hex):");
    println!("{}", signature);
    Ok(())
}
