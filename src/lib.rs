use std::ffi::OsStr;
use std::process::{Command};
use std::str::FromStr;
use std::thread;
use log::{trace};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct InternalEventHandler {
    pub name: String,
    pub path: Option<Regex>,
    pub path_not: Option<bool>,
    pub member: Option<Regex>,
    pub member_not: Option<bool>,
    pub data: Option<Regex>,
    pub data_not: Option<bool>,
    pub exec: Option<String>,
    pub signal: Option<u32>,
    pub signal_process: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EventHandler {
    pub path: Option<String>,
    pub path_not: Option<bool>,
    pub member: Option<String>,
    pub member_not: Option<bool>,
    pub data: Option<String>,
    pub data_not: Option<bool>,
    pub exec: Option<String>,
    pub signal: Option<u32>,
    pub signal_process: Option<String>,
}
impl From<EventHandler> for InternalEventHandler {
    fn from(val: EventHandler) -> Self {
        InternalEventHandler {
            name: "".to_string(),
            path: val.path.map(|e| Regex::from_str(&e).expect("path regex error")),
            path_not: val.path_not.map(|e| e),
            member: val.member.map(|e| Regex::from_str(&e).expect("member regex error")),
            member_not: val.member_not.map(|e| e),
            data: val.data.map(|e| Regex::from_str(&e).expect("data regex error")),
            data_not: val.data_not.map(|e| e),
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
            path_not: val.1.path_not.map(|e| e),
            member: val.1.member.map(|e| Regex::from_str(&e).expect("member regex error")),
            member_not: val.1.member_not.map(|e| e),
            data: val.1.data.map(|e| Regex::from_str(&e).expect("data regex error")),
            data_not: val.1.data_not.map(|e| e),
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
        trace!("Process not active")
    }
}
pub fn run_shell_command(handler_name: String, command: String) {
    trace!("Run Shell Command: {}", command);
    let _ = thread::spawn( move || {
        let result = Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .status();
        trace!("{} Command exited with exit code: {}", handler_name, result.expect("command failed").code().unwrap_or(-1));
    });
}

