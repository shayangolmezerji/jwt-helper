use std::io::BufRead;
use anyhow::Context;
use std::fs::File;
use std::io::BufReader;
use anyhow::Result;


pub fn batch_command(file: &str) -> Result<()> {
    println!("[Batch] Processing batch file: {}", file);
        let f = File::open(file).with_context(|| format!("Failed to open batch file: {}", file))?;
    let reader = BufReader::new(f);
    for (i, line) in reader.lines().enumerate() {
            let token = line?;
            let token = token.as_str();
        println!("\n=== Token {} ===", i + 1);
        match crate::commands::decode::decode_jwt(&token) {
            Ok(_) => {},
            Err(e) => println!("Error decoding token: {}", e),
        }
    }

    Ok(())
}
