use anyhow::Error;
use btinfo::{notify_process, run_shell_command, EventHandler, InternalEventHandler};
use log::{debug, trace, warn};
use std::collections::HashMap;
use std::fs::File;
use tokio::fs;
use zbus::export::ordered_stream::OrderedStreamExt;
use zbus::fdo::DBusProxy;
use zbus::message::Type;
use zbus::{Connection, MatchRule, Message, MessageStream};
use zvariant::{Structure};

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

    let toml: Vec<InternalEventHandler> = toml::from_str::<HashMap<String, EventHandler>>(&config)?
        .into_iter()
        .map(|e| e.into())
        .collect();

    let connection = Connection::session().await?;

    let dbus_proxy = DBusProxy::new(&connection).await?;

    dbus_proxy
        .add_match_rule(MatchRule::try_from("type='signal'")?)
        .await?;

    println!("Listening to all D-Bus signals...");

    let mut stream = MessageStream::from(&connection);

    while let Some(msg) = stream.next().await {
        
        let (msg, data) = parse_signal(msg?)?;
        
        if msg.message_type() == Type::Signal {
            if data.len() == 0 {
                trace!(
                    "{}_{}",
                    msg.header().path().expect("path"),
                    msg.header().member().expect("member")
                );
            } else {
                trace!(
                    "{}_{}_\n{}",
                    msg.header().path().expect("path"),
                    msg.header().member().expect("member"),
                    data
                );
            }

            for handler in &toml {
                if matches_config_rule(&msg, &data, handler)
                {
                    if let Some(signal) = handler.signal {
                        send_signal(&handler, signal);
                    }

                    if let Some(exec) = &handler.exec {
                        let result = run_shell_command(exec).expect("status code");
                        trace!(
                            "{} Command exited with exit code: {}",
                            handler.name,
                            result
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn send_signal(handler: &&InternalEventHandler, signal: u32) {
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

fn matches_config_rule(msg: &Message, data: &String, handler: &InternalEventHandler) -> bool {
    handler
        .path
        .as_ref()
        .map(|e| e.is_match(msg.header().path().expect("path")))
        .unwrap_or(true)
        && handler
        .member
        .as_ref()
        .map(|e| e.is_match(msg.header().member().expect("member")))
        .unwrap_or(true)
        && handler
        .data
        .as_ref()
        .map(|e| e.is_match(&data))
        .unwrap_or(true)
}

fn parse_signal(msg: Message) -> Result<(Message, String), Error> {

    let body = msg.body();
    let body = body
        .deserialize::<Structure>()
        .map(|e| Some(e))
        .map_or_else(|e| Option::<Structure>::None, |e| e);

    let data = if let Some(b) = body {
        let content: Vec<String> = b
            .into_fields()
            .into_iter()
            .map(|e| e.try_to_owned().unwrap())
            .map(|ee| serde_json::to_string_pretty(&ee).unwrap())
            .collect();
        content.join(",\n")
    } else {
        "".to_string()
    };
    Ok((msg, data))
}
