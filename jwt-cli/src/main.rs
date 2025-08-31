use clap::{Parser, Subcommand, Args, ValueEnum};
use anyhow::Result;
mod commands;
use commands::{
    jwe::{jwe_encrypt_command, jwe_decrypt_command},
    build::build_jwt,
    decode::decode_jwt,
    verify::verify_jwt,
    keys::{generate_rsa, generate_ec, generate_hmac},
    batch::batch_command,
    fuzz::fuzz_command,
    visualize::visualize_command,
    jwks::jwks_command,
    jwk::jwk_command,
    claim_edit::claim_edit_command,
    vulnscan::vulnscan_command,
    diff::diff_command,
    expcheck::expcheck_command,
    oidc::oidc_command,
};

#[derive(Parser)]
#[command(name = "jwt")]
#[command(about = "A powerful CLI tool for JWT operations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    
    Build(BuildArgs),
    
    Decode {
        
        token: String,
    },
    
    Verify(VerifyArgs),
    
    #[command(subcommand)]
    Keys(KeysCommand),
    
    Batch {
        
        file: String,
    },
    
    Fuzz {
        
        token: String,
        
        wordlist: String,
        
        alg: String,
    },
    
    Jwks {
        
        url: String,
        
        token: String,
    },
    
    Jwk {
        
        input: String,
        
        #[arg(long)]
        to_pem: bool,
    },
    
    ClaimEdit {
        
        token: String,
        
        edits: String,
        
        secret: String,
        
        alg: String,
    },
    
    VulnScan {
        
        token: String,
    },
    
    Diff {
        
        token1: String,
        
        token2: String,
    },
    
    ExpCheck {
        
        token: String,
        
        at_time: Option<String>,
    },
    
    Visualize {
        
        token: String,
        
        mode: String,
    },
    
    Oidc {
        
        endpoint: String,
        
        token: String,
    },
    
    JweEncrypt {
        
        payload: String,
        
        key: String,
        
        alg: String,
    },
    
    JweDecrypt {
        
        token: String,
        
        key: String,
    },
}

#[derive(Args)]
struct BuildArgs {
    #[arg(long, value_enum)]
    alg: Algorithm,
    #[arg(long)]
    secret: Option<String>,
    #[arg(long, value_name = "PATH")]
    private_key: Option<String>,
    #[arg(long)]
    payload: Option<String>,
    #[arg(long, value_name = "PATH")]
    payload_file: Option<String>,
    #[arg(long)]
    exp: Option<String>,
}

#[derive(Args)]
struct VerifyArgs {
    token: String,
    #[arg(long, value_enum)]
    alg: Algorithm,
    #[arg(long)]
    secret: Option<String>,
    #[arg(long, value_name = "PATH")]
    public_key: Option<String>,
}

#[derive(Subcommand)]
enum KeysCommand {
    
    GenerateRsa {
        #[arg(long, default_value_t = 2048)]
        size: u32,
    },
    
    GenerateEc {
        #[arg(long, default_value = "P-256")]
        curve: String,
    },
    
    GenerateHmac {
        #[arg(long, default_value_t = 256)]
        size: u32,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Algorithm {
    HS256,
    HS384,
    HS512,
    RS256,
    RS384,
    RS512,
    ES256,
    ES384,
    EdDSA,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build(args) => build_jwt(args)?,
        Command::Decode { token } => decode_jwt(&token)?,
        Command::Verify(args) => verify_jwt(args)?,
        Command::Keys(cmd) => match cmd {
            KeysCommand::GenerateRsa { size } => generate_rsa(size)?,
            KeysCommand::GenerateEc { curve } => generate_ec(&curve)?,
            KeysCommand::GenerateHmac { size } => generate_hmac(size)?,
        },
        Command::Batch { file } => batch_command(&file)?,
        Command::Fuzz { token, wordlist, alg } => fuzz_command(&token, &wordlist, &alg)?,
        Command::Jwks { url, token } => jwks_command(&url, &token)?,
        Command::Jwk { input, to_pem } => jwk_command(&input, to_pem)?,
        Command::ClaimEdit { token, edits, secret, alg } => claim_edit_command(&token, &edits, &secret, &alg)?,
        Command::VulnScan { token } => vulnscan_command(&token)?,
        Command::Diff { token1, token2 } => diff_command(&token1, &token2)?,
        Command::ExpCheck { token, at_time } => expcheck_command(&token, at_time.as_deref())?,
        Command::Visualize { token, mode } => visualize_command(&token, &mode)?,
        Command::Oidc { endpoint, token } => oidc_command(&endpoint, &token)?,
        Command::JweEncrypt { payload, key, alg } => jwe_encrypt_command(&payload, &key, &alg)?,
        Command::JweDecrypt { token, key } => jwe_decrypt_command(&token, &key)?,
    }
    Ok(())
}


