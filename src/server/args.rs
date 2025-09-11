use clap::{
    Parser
};
use std::env;

fn default_identity_path() -> String {
    let home_dir = env::var("HOME").unwrap_or("/".to_string());
    return format!("{}/.config/CruxC2/identity.pfx", home_dir)
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
#[command(name = "CruxServer")]
#[command(author = "Brian Cao [https://haoyingcao.xyz]")]
#[command(version = "0.1")]
#[command(about = "A simple Command & Control Server inside the CruxC2 framework.", long_about = None)]

pub struct CruxServerArgs {
    /// The port to listen on
    #[arg(short = 'p', long = "port", default_value_t = String::from("1337"))]
    pub port: String,
    /// PKCS12 Identity File path
    #[arg(short = 'f', long = "pkcs12-path", default_value_t = default_identity_path())]
    pub identity: String
}
