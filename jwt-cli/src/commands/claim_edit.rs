use anyhow::Result;
use serde_json::Value;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};


pub fn claim_edit_command(token: &str, edits: &str, secret: &str, alg: &str) -> Result<()> {
    let alg = match alg.to_ascii_uppercase().as_str() {
        "HS256" => Algorithm::HS256,
        "HS384" => Algorithm::HS384,
        "HS512" => Algorithm::HS512,
        _ => anyhow::bail!("Only HS256/384/512 supported for claim edit"),
    };
    let validation = Validation::new(alg);
    let token_data = decode::<Value>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)?;
    let mut claims = token_data.claims;
    let edits: Value = serde_json::from_str(edits)?;
    if let Value::Object(edits_map) = edits {
        for (k, v) in edits_map {
            claims[k] = v;
        }
    }
    let header = Header::new(alg);
    let new_token = encode(&header, &claims, &EncodingKey::from_secret(secret.as_bytes()))?;
    println!("{}", new_token);
    Ok(())
}
