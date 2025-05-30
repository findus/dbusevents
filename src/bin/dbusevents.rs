use log::{debug, trace, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::str::FromStr;
use futures_util::FutureExt;
use tokio::fs;
use toml::Value;
use zbus::export::ordered_stream::OrderedStreamExt;
use zbus::fdo::DBusProxy;
use zbus::message::Type;
use zbus::{Connection, MatchRule, MessageStream};
use zvariant::{signature, Array, DynamicType, OwnedValue, Structure};
use zvariant::Signature::Signature;
use btinfo::{notify_process, run_shell_command};

struct InternalEventHandler {
    name: String,
    path: Regex,
    member: Regex,
    exec: Option<String>,
    signal: Option<u32>,
    signal_process: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct EventHandler {
    path: String,
    member: String,
    exec: Option<String>,
    signal: Option<u32>,
    signal_process: Option<String>,
}
impl From<EventHandler> for InternalEventHandler {
    fn from(val: EventHandler) -> Self {
        InternalEventHandler {
            name: "".to_string(),
            path: Regex::from_str(&val.path).expect("path regex error"),
            member: Regex::from_str(&val.member).expect("path regex error"),
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
            path: Regex::from_str(&val.1.path).expect("path regex error"),
            member: Regex::from_str(&val.1.member).expect("path regex error"),
            exec: val.1.exec,
            signal: val.1.signal,
            signal_process: val.1.signal_process,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut path = xdg::BaseDirectories::new()
        .config_home
        .expect("config home");
    let path = path.as_mut_os_string();
    path.push("/dbuseventshandler");

    fs::create_dir_all(&path).await?;
    path.push("/config.toml");
    trace!("{:?}", path);
    if let Ok(false) = fs::try_exists(&path).await {
        File::create(&path).expect("Could not create file");
    }

    let config = fs::read_to_string(path).await?;

    if config.is_empty() {
        warn!("Config is empty, exiting.");
        return Ok(());
    }

    let toml: Vec<InternalEventHandler> =
        toml::from_str::<HashMap<String, EventHandler>>(&config)?
            .into_iter()
            .map(|e| e.into())
            .collect();

    // Connect to the session bus (use `Connection::system()` for system bus)
    let connection = Connection::session().await?;

    // Get a proxy to the D-Bus service to add a match rule
    let dbus_proxy = DBusProxy::new(&connection).await?;

    // Add a match rule to receive all signals
    dbus_proxy
        .add_match_rule(MatchRule::try_from("type='signal'")?)
        .await?;

    println!("Listening to all D-Bus signals...");

    // Create a MessageStream to receive messages
    let mut stream = MessageStream::from(&connection);

    // Process incoming messages
    while let Some(msg) = stream.next().await {
        let msg = msg?;

        let body = msg.body();
        let sig  = body.signature();
        let body = body
            .deserialize::<Structure>().map(|e| Some(e))
            .map_or_else(|e| Option::<Structure>::None, |e|e);

        let lol = if let Some(b) = body {
            let fields: Vec<OwnedValue> = b.into_fields()
                .into_iter().map(|e| e.try_to_owned().unwrap())
                .collect();
            let lolll: Vec<String> = fields.into_iter().map(|ee| {

            }).collect();
            print!("{:?}", lolll);
        } else {
            ()
        };


        //let fields = body.into_fields();

        if msg.message_type() == Type::Signal {
            trace!(
                "{}_{}_{:?}",
                msg.header().path().expect("path"),
                msg.header().member().expect("member"),
                lol
            );

            for handler in &toml {
                if handler.path.is_match(msg.header().path().expect("path")) && handler
                        .member
                        .is_match(msg.header().member().expect("member")) {
                    
                    if let Some(signal) = handler.signal {
                        let proc = &handler.signal_process;
                        let proc = proc
                            .as_ref()
                            .expect("executable to send signal to not found");
                        debug!(
                            "[{}] Notify: {} with Signal: {}",
                            handler.name, proc, signal
                        );
                        notify_process(proc, signal as i32);
                    }
                    
                    if let Some(exec) = &handler.exec {
                        trace!("{} Command exited with exit code: {}",handler.name, run_shell_command(exec).expect("status code"));
                    }
                }
            }
        }
    }

    Ok(())
}
fn to_str(owned: OwnedValue) -> String {

    match owned.signature() {
        signature::Signature::Array(child) => {
            child.
        }
        signature::Signature::Unit => {}
        signature::Signature::U8 => {}
        signature::Signature::Bool => {}
        signature::Signature::I16 => {}
        signature::Signature::U16 => {}
        signature::Signature::I32 => {}
        signature::Signature::U32 => {}
        signature::Signature::I64 => {}
        signature::Signature::U64 => {}
        signature::Signature::F64 => {}
        signature::Signature::Str => {}
        Signature => {}
        signature::Signature::ObjectPath => {}
        signature::Signature::Variant => {}
        signature::Signature::Fd => {}
        signature::Signature::Dict { .. } => {}
        signature::Signature::Structure(_) => {}
    }
}