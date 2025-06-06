use std::ffi::OsStr;
use std::process::{Command, ExitStatus};
use std::io::{Error};
use std::str::FromStr;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct InternalEventHandler {
    pub name: String,
    pub path: Option<Regex>,
    pub member: Option<Regex>,
    pub data: Option<Regex>,
    pub exec: Option<String>,
    pub signal: Option<u32>,
    pub signal_process: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EventHandler {
    pub path: Option<String>,
    pub member: Option<String>,
    pub data: Option<String>,
    pub exec: Option<String>,
    pub signal: Option<u32>,
    pub signal_process: Option<String>,
}
impl From<EventHandler> for InternalEventHandler {
    fn from(val: EventHandler) -> Self {
        InternalEventHandler {
            name: "".to_string(),
            path: val.path.map(|e| Regex::from_str(&e).expect("path regex error")),
            member: val.member.map(|e| Regex::from_str(&e).expect("member regex error")),
            data: val.data.map(|e| Regex::from_str(&e).expect("data regex error")),
            exec: val.exec,
            signal: val.signal,
            signal_process: val.signal_process,
        }
    }
}

impl From<(String, EventHandler)> for InternalEventHandler {
    fn from(val: (String, EventHandler)) -> Self {
        InternalEventHandler {
            name: val.0,
            path: val.1.path.map(|e| Regex::from_str(&e).expect("path regex error")),
            member: val.1.member.map(|e| Regex::from_str(&e).expect("member regex error")),
            data: val.1.data.map(|e| Regex::from_str(&e).expect("data regex error")),
            exec: val.1.exec,
            signal: val.1.signal,
            signal_process: val.1.signal_process,
        }
    }
}

pub fn notify_process(process: &str, signal: i32) {
    let mut system = sysinfo::System::new();
    system.refresh_all();

    let pid = system
        .processes_by_exact_name(OsStr::new(process))
        .next()
        .map(|e| e.pid().as_u32() as i32);

    if let Some(pid) = pid {
        let signal_number = { libc::SIGRTMIN() + signal };
        let _ = unsafe { libc::kill(pid, signal_number) };
    } else {
        println!("Waybar not active")
    }
}
pub fn run_shell_command(command: &str) -> Result<ExitStatus, Error> {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .status()
}

