use anyhow::Result;
use jsonwebtoken::decode_header;


pub fn vulnscan_command(token: &str) -> Result<()> {
    let header = decode_header(token)?;

    if format!("{:?}", header.alg).to_lowercase() == "none" {
        println!("[!] Vulnerability: alg:none detected");
    }
    if let Some(kid) = header.kid {
        if kid.is_empty() {
            println!("[!] Vulnerability: empty kid");
        }
    }

    println!("Scan complete.");
    Ok(())
}
