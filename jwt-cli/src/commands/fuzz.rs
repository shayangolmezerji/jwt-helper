use anyhow::{Result, Context};
use std::fs::File;
use std::io::{BufRead, BufReader};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::Value;


pub fn fuzz_command(token: &str, wordlist: &str, alg: &str) -> Result<()> {
    let alg = match alg.to_ascii_uppercase().as_str() {
        "HS256" => Algorithm::HS256,
        "HS384" => Algorithm::HS384,
        "HS512" => Algorithm::HS512,
        _ => anyhow::bail!("Only HS256/384/512 supported for brute-force"),
    };
    let f = File::open(wordlist).with_context(|| format!("Failed to open wordlist: {}", wordlist))?;
    let reader = BufReader::new(f);
    let mut found = false;
    for (i, line) in reader.lines().enumerate() {
        let secret = line?;
        let res = decode::<Value>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::new(alg));
        if res.is_ok() {
            println!("[+] Secret found (line {}): {}", i + 1, secret);
            found = true;
        }
    }
    if !found {
        println!("[-] No valid secret found in wordlist");
    }
    Ok(())
}
