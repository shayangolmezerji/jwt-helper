use anyhow::{Result, Context};
use std::fs;
use jsonwebtoken::jwk::{Jwk, AlgorithmParameters};


pub fn jwk_command(input: &str, to_pem: bool) -> Result<()> {
    let data = fs::read_to_string(input).with_context(|| format!("Failed to read input: {}", input))?;
    if to_pem {
        let jwk: Jwk = serde_json::from_str(&data)?;
        match jwk.algorithm {
            AlgorithmParameters::RSA(ref _rsa) => {

            }
            _ => anyhow::bail!("Only RSA JWK to PEM supported"),
        }
    } else {

        anyhow::bail!("PEM to JWK not implemented yet");
    }
    Ok(())
}
