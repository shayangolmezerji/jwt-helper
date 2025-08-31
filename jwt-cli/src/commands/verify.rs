use anyhow::Result;

pub fn verify_jwt(args: crate::VerifyArgs) -> Result<()> {
    use jsonwebtoken::{decode, Algorithm as JwtAlgorithm, DecodingKey, Validation, TokenData};
    use std::fs;
    use chrono::Utc;
    use serde_json::Value;
    use anyhow::{bail, Context};

    let alg = match args.alg {
        crate::Algorithm::HS256 => JwtAlgorithm::HS256,
        crate::Algorithm::HS384 => JwtAlgorithm::HS384,
        crate::Algorithm::HS512 => JwtAlgorithm::HS512,
        crate::Algorithm::RS256 => JwtAlgorithm::RS256,
        crate::Algorithm::RS384 => JwtAlgorithm::RS384,
        crate::Algorithm::RS512 => JwtAlgorithm::RS512,
        crate::Algorithm::ES256 => JwtAlgorithm::ES256,
        crate::Algorithm::ES384 => JwtAlgorithm::ES384,
        crate::Algorithm::EdDSA => JwtAlgorithm::EdDSA,
    // _ => bail!("Unsupported algorithm for verify"),
    };

    let key = match alg {
        JwtAlgorithm::HS256 | JwtAlgorithm::HS384 | JwtAlgorithm::HS512 => {
            let secret = args.secret.as_ref().context("--secret is required for HS* algorithms")?;
            DecodingKey::from_secret(secret.as_bytes())
        }
        JwtAlgorithm::RS256 | JwtAlgorithm::RS384 | JwtAlgorithm::RS512 => {
            let key_path = args.public_key.as_ref().context("--public-key is required for RS* algorithms")?;
            let key = fs::read(key_path).with_context(|| format!("Failed to read public key: {}", key_path))?;
            DecodingKey::from_rsa_pem(&key).context("Invalid RSA public key")?
        }
        JwtAlgorithm::ES256 | JwtAlgorithm::ES384 => {
            let key_path = args.public_key.as_ref().context("--public-key is required for ES* algorithms")?;
            let key = fs::read(key_path).with_context(|| format!("Failed to read public key: {}", key_path))?;
            DecodingKey::from_ec_pem(&key).context("Invalid EC public key")?
        }
        JwtAlgorithm::EdDSA => {
            let key_path = args.public_key.as_ref().context("--public-key is required for EdDSA algorithm")?;
            let key = fs::read(key_path).with_context(|| format!("Failed to read public key: {}", key_path))?;
            DecodingKey::from_ed_pem(&key).context("Invalid EdDSA public key")?
        }
        _ => bail!("Unsupported algorithm for verify key"),
    };

    let mut validation = Validation::new(alg);
    validation.validate_exp = true;

    let token_data: TokenData<Value> = decode::<Value>(&args.token, &key, &validation)
        .map_err(|e| anyhow::anyhow!("JWT verification failed: {}", e))?;

    println!("Signature: valid");
    if let Some(exp) = token_data.claims.get("exp").and_then(|v| v.as_i64()) {
        let now = Utc::now().timestamp();
        if exp < now {
            bail!("Token is expired (exp: {}, now: {})", exp, now);
        }
    }
    println!("Claims:");
    serde_json::to_writer_pretty(std::io::stdout(), &token_data.claims)?;
    println!();
    Ok(())
}
