use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum CmdType {
    Exit,
    Cd,
    Setvar,
    Export,
    Download,
    Upload,
    Exec,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cmd{
    pub cmd_type: CmdType,

    // Argument for the CMD:
    // Exit: Empty
    // Cd, Download/Upload: path
    // Exec: Command to be executed
    pub args: String
}


#[derive(Serialize, Deserialize, Debug)]
pub struct CmdResult {
    pub status: i64,
    pub output: String // Combined stdout stderr stream, preserving order
}

impl Default for CmdResult {
    fn default() -> Self {
        Self {
            status: -1,
            output: "".to_string()
        }
    }
}
