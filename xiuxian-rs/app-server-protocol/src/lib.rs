use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Request {
    pub id: u64,
    pub command: Command,
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    StartDispatch,
    StopDispatch,
    SetClicker {
        enabled: bool,
    },
    ConfigureClicker {
        interval: u64,
        right: bool,
        key: i32,
    },
    Shutdown,
}
#[derive(Serialize, Clone, PartialEq)]
pub struct LogEntry {
    pub id: u64,
    pub time: String,
    pub message: String,
}
#[derive(Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClickerState {
    pub enabled: bool,
    pub count: u64,
    pub status: String,
    pub interval: u64,
    pub right: bool,
    pub key: i32,
}
#[derive(Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub dispatch_running: bool,
    pub dispatch_stopping: bool,
    pub dispatch_status: String,
    pub map: String,
    pub success: u64,
    pub failure: u64,
    pub game_connected: bool,
    pub clicker: ClickerState,
    pub logs: Vec<LogEntry>,
    pub closing: bool,
}
