use std::fmt;
use std::vec;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum OS {
    Windows,
    Linux,
    Unknown
}

impl fmt::Display for OS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            OS::Windows => "Windows",
            OS::Linux => "Linux",
            OS::Unknown => "Unknown"
        };
        write!(f, "{}", text)
    }
}

pub fn os_detect() -> OS {
    if cfg!(target_os = "windows"){
        OS::Windows
    }
    else if cfg!(target_os = "linux"){
        OS::Linux
    }
    else {
        OS::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub username: String,
    pub hostname: String,
    pub os_type: OS,
    pub shell_path: String
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    // Server -> Agent
    // Argument for the Cmd:
    // Exit: Empty
    // Cd, Download/Upload: path
    // Exec: Command to be executed
    Cmd {id: u64, cmd_type: CmdType, args: String},
    Kill {id: u64},
    // Agent -> Server
    CmdOutput {id: u64, data: Vec<u8>},
    CmdExit {id: u64, status: i64},
    CmdError {id: u64, error: String}
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CmdType {
    Exit,
    Cd,
    Export,
    Download,
    Upload,
    Exec,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Cmd{
//     pub cmd_type: CmdType,
//
//     pub args: String
// }
//
//
// #[derive(Serialize, Deserialize, Debug)]
// pub struct CmdResult {
//     pub status: i64,
//     pub output: String // Combined stdout stderr stream, preserving order
// }
//
// impl Default for CmdResult {
//     fn default() -> Self {
//         Self {
//             status: -1,
//             output: "".to_string()
//         }
//     }
// }
