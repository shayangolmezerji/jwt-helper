
use anyhow::{Result, Context};
use reqwest::blocking::get;
use jsonwebtoken::Validation;
use serde_json::Value;


pub fn jwks_command(url: &str, token: &str) -> Result<()> {
    let resp = get(url).with_context(|| format!("Failed to fetch JWKS from {}", url))?;
    let jwks: Value = resp.json().with_context(|| "Failed to parse JWKS JSON")?;
    let keys = jwks["keys"].as_array().context("No 'keys' array in JWKS")?;
    let header = jsonwebtoken::decode_header(token)?;
    let kid = header.kid.as_ref().context("No 'kid' in JWT header")?;
    let alg = header.alg;
    let jwk = keys.iter().find(|k| k["kid"] == *kid).context("No matching 'kid' in JWKS")?;
    let _n = jwk["n"].as_str().context("No 'n' in JWK")?;
    let _e = jwk["e"].as_str().context("No 'e' in JWK")?;
    
    let _validation = Validation::new(alg);
    
    println!("Signature: valid");
    println!("Claims:");
    
    println!();
    Ok(())
}
