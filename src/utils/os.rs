use std::fmt;
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

// pub fn parse_os(input: &str) -> OS {
//     match input.trim() {
//         "Windows" => return OS::Windows,
//         "Linux" => return OS::Linux,
//         _ => return OS::Unknown
//     };
// }

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
    pub os_type: OS
}
