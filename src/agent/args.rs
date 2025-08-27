use clap::{
    Parser
};
use std::net::IpAddr;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
#[command(name = "CruxAgent")]
#[command(author = "Brian Cao [https://haoyingcao.xyz]")]
#[command(version = "0.1")]
#[command(about = "A simple Command & Control Agent inside the CruxC2 framework.", long_about = None)]

pub struct CruxAgentArgs {
    /// Remote port to connect to (short -p)
    #[arg(short = 'p', long = "port", default_value_t = String::from("1337"))]
    pub rport: String,

    /// Remote host to connect to (mandatory)
    pub rhost: IpAddr
}
