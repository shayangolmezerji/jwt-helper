use anyhow::Result;

pub fn generate_rsa(size: u32) -> Result<()> {
    use rsa::{RsaPrivateKey, pkcs8::{EncodePrivateKey, EncodePublicKey}};
    use rand::rngs::OsRng;
    use anyhow::Context;
    let mut rng = OsRng;
    let private = RsaPrivateKey::new(&mut rng, size as usize)
        .with_context(|| format!("Failed to generate RSA-{} key", size))?;
    let public = private.to_public_key();
    let priv_pem = private.to_pkcs8_pem(Default::default())?.to_string();
    let pub_pem = public.to_public_key_pem(Default::default())?;
    println!("-----BEGIN PRIVATE KEY-----\n{}-----END PRIVATE KEY-----", priv_pem.trim_matches('\n'));
    println!("\n-----BEGIN PUBLIC KEY-----\n{}-----END PUBLIC KEY-----", pub_pem.trim_matches('\n'));
    Ok(())
}

pub fn generate_ec(curve: &str) -> Result<()> {
    use p256::ecdsa::SigningKey;
    use p256::pkcs8::{EncodePrivateKey, EncodePublicKey};
    use rand::rngs::OsRng;
    use anyhow::bail;
    match curve {
        "P-256" => {
            let signing_key = SigningKey::random(&mut OsRng);
            let verify_key = signing_key.verifying_key();
            let priv_pem = signing_key.to_pkcs8_pem(Default::default())?.to_string();
            let pub_pem = verify_key.to_public_key_pem(Default::default())?;
            println!("-----BEGIN PRIVATE KEY-----\n{}-----END PRIVATE KEY-----", priv_pem.trim_matches('\n'));
            println!("\n-----BEGIN PUBLIC KEY-----\n{}-----END PUBLIC KEY-----", pub_pem.trim_matches('\n'));
        }
        _ => bail!("Unsupported curve: {}. Only P-256 is supported.", curve),
    }
    Ok(())
}

pub fn generate_hmac(size: u32) -> Result<()> {
    use rand::RngCore;
    use zeroize::Zeroize;
    let nbytes = (size + 7) / 8;
    let mut key = vec![0u8; nbytes as usize];
    rand::thread_rng().fill_bytes(&mut key);
    println!("HMAC secret (hex): {}", hex::encode(&key));
    key.zeroize();
    Ok(())
}
