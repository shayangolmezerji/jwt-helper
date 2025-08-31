use anyhow::Result;

pub fn build_jwt(args: crate::BuildArgs) -> Result<()> {
    use jsonwebtoken::{encode, Algorithm as JwtAlgorithm, EncodingKey, Header};
    use std::fs;
    use serde_json::Map;
    use zeroize::Zeroize;
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
    
    };

    let mut payload: Map<String, serde_json::Value> = if let Some(ref file) = args.payload_file {
        let data = fs::read_to_string(file).with_context(|| format!("Failed to read payload file: {}", file))?;
        serde_json::from_str(&data).with_context(|| "Invalid JSON in payload file")?
    } else if let Some(ref s) = args.payload {
        serde_json::from_str(s).with_context(|| "Invalid JSON in --payload")?
    } else {
        Map::new()
    };

    if let Some(ref exp_str) = args.exp {
    let exp = super::parse_exp(exp_str)?;
        payload.insert("exp".to_string(), serde_json::json!(exp));
    }

    let mut header = Header::new(alg);
    header.typ = Some("JWT".to_string());

    let token = match alg {
        JwtAlgorithm::HS256 | JwtAlgorithm::HS384 | JwtAlgorithm::HS512 => {
            let mut secret = args.secret.as_ref().context("--secret is required for HS* algorithms")?.as_bytes().to_vec();
            let token = encode(&header, &payload, &EncodingKey::from_secret(&secret))?;
            secret.zeroize();
            token
        }
        JwtAlgorithm::RS256 | JwtAlgorithm::RS384 | JwtAlgorithm::RS512 => {
            let key_path = args.private_key.as_ref().context("--private-key is required for RS* algorithms")?;
            let key = fs::read(key_path).with_context(|| format!("Failed to read private key: {}", key_path))?;
            encode(&header, &payload, &EncodingKey::from_rsa_pem(&key).context("Invalid RSA private key")?)?
        }
        JwtAlgorithm::ES256 | JwtAlgorithm::ES384 => {
            let key_path = args.private_key.as_ref().context("--private-key is required for ES* algorithms")?;
            let key = fs::read(key_path).with_context(|| format!("Failed to read private key: {}", key_path))?;
            encode(&header, &payload, &EncodingKey::from_ec_pem(&key).context("Invalid EC private key")?)?
        }
        JwtAlgorithm::EdDSA => {
            let key_path = args.private_key.as_ref().context("--private-key is required for EdDSA algorithm")?;
            let key = fs::read(key_path).with_context(|| format!("Failed to read private key: {}", key_path))?;
            encode(&header, &payload, &EncodingKey::from_ed_pem(&key).context("Invalid EdDSA private key")?)?
        }
        _ => bail!("Unsupported algorithm for build token"),
    };

    println!("{}", token);
    Ok(())
}
