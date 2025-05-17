use std::collections::HashMap;
use std::fs::File;
use std::str::FromStr;
use lazy_static::lazy_static;
use libc::exit;
use zbus::{Connection, MatchRule, Message, MessageStream};
use zbus::export::ordered_stream::OrderedStreamExt;
use zbus::fdo::DBusProxy;
use zbus::message::Type;
use zvariant::Value;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::fs;

struct InternalEventHandler {
    name: String,
    path: Regex,
    member: Regex,
    exec: Option<String>,
    signal: Option<u32>
}

#[derive(Serialize, Deserialize)]
struct EventHandler {
    path: String,
    member: String,
    exec: Option<String>,
    signal: Option<u32>
}
impl Into<InternalEventHandler> for EventHandler {
    fn into(self) -> InternalEventHandler {
        InternalEventHandler {
            name: "".to_string(),
            path: Regex::from_str(&self.path).expect("path regex error"),
            member: Regex::from_str(&self.member).expect("path regex error"),
            exec: self.exec,
            signal: self.signal
        }
    }
}

impl Into<InternalEventHandler> for (String, EventHandler) {
    fn into(self) -> InternalEventHandler {
        InternalEventHandler {
            name: self.0,
            path: Regex::from_str(&self.1.path).expect("path regex error"),
            member: Regex::from_str(&self.1.member).expect("path regex error"),
            exec: self.1.exec,
            signal: self.1.signal
        }
    }
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let mut path = xdg::BaseDirectories::new().config_home.expect("config home");
    let mut path = path.as_mut_os_string();
    path.push("/dbuseventshandler");

    fs::create_dir_all(&path).await?;
    path.push("/config.toml");
    println!("{:?}", path);
    if let Ok(false) = fs::try_exists(&path).await {
        File::create(&path).expect("Could not create file");
    }
    
    let config = fs::read_to_string(path).await?;
    
    if config.is_empty() {
        return Ok(());
    }
    
    let toml: Vec<InternalEventHandler> = toml::from_str::<HashMap<String,EventHandler>>(&*config)?
        .into_iter()
        .map(|e|e.into())
        .collect();
        
    // Connect to the session bus (use `Connection::system()` for system bus)
    let connection = Connection::session().await?;

    // Get a proxy to the D-Bus service to add a match rule
    let dbus_proxy = DBusProxy::new(&connection).await?;

    // Add a match rule to receive all signals
    dbus_proxy.add_match_rule(MatchRule::try_from("type='signal'")?).await?;

    println!("Listening to all D-Bus signals...");

    // Create a MessageStream to receive messages
    let mut stream = MessageStream::from(&connection);

    // Process incoming messages
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        if msg.message_type() == Type::Signal {
            println!(
                "{}_{}",
                msg.header().path().expect("path"),
                msg.header().member().expect("member")
            );

           // println!("Body: {:#?}", msg.body());
            println!("---");
        }
    }

    Ok(())
}