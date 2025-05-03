use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zbus::zvariant::{ObjectPath, OwnedValue};
use zbus::Connection;
use zvariant::OwnedObjectPath;

pub type BluezManagedObjects = HashMap<
    OwnedObjectPath, // object path
    HashMap<
        String, // interface name
        HashMap<
            String,     // property name
            OwnedValue, // variant value
        >,
    >,
>;

#[derive(serde::Serialize, Deserialize, Debug)]
struct BluetoothStatus {
    bat: u8,
    name: String,
    btype: String,
    address: String,
}

async fn get_devices() -> Result<Vec<BluetoothStatus>, Error> {
    let connection = Connection::system().await?;
    let proxy = zbus::Proxy::new(
        &connection,
        "org.bluez",
        "/",
        "org.freedesktop.DBus.ObjectManager",
    )
    .await?;

    let objects: BluezManagedObjects = proxy.call("GetManagedObjects", &()).await?;
    let mut vec: Vec<BluetoothStatus> = vec![];

    for (_, interfaces) in objects {
        if let Some(device_props) = interfaces.get("org.bluez.Device1") {
            if let Some(connected_val) = device_props.get("Connected") {
                if let Ok(true) = connected_val.downcast_ref::<bool>() {
                    let name = device_props
                        .get("Name")
                        .and_then(|v| v.downcast_ref::<String>().into())
                        .unwrap()?;

                    let address = device_props
                        .get("Address")
                        .and_then(|v| v.downcast_ref::<String>().into())
                        .unwrap()?;

                    let iconType = device_props
                        .get("Icon")
                        .and_then(|v| v.downcast_ref::<String>().into())
                        .unwrap()?;

                    let icon = match iconType.as_str() {
                        "input-mouse" => "".to_string(),
                        "input-keyboard" => "".to_string(),
                        "audio-headset" | "audio-headphones" => "".to_string(),
                        _ => iconType,
                    };

                    // Try to get battery level if available
                    let battery_level = interfaces
                        .get("org.bluez.Battery1")
                        .and_then(|battery_props| {
                            battery_props
                                .get("Percentage")
                                .and_then(|v| v.downcast_ref::<u8>().into())
                        })
                        .unwrap();

                    vec.push(BluetoothStatus {
                        bat: battery_level.unwrap(),
                        name,
                        btype: icon,
                        address,
                    });
                }
            }
        }
    }

    return Ok(vec);
}

#[derive(serde::Serialize, Deserialize, Debug)]
pub struct WaybarStatus {
    text: String,
    class: String,
}

pub fn format_waybar(devices: &Vec<BluetoothStatus>) -> WaybarStatus {
    let text = devices.iter().fold("".to_string(), |last, entry| {
        let batperc = match entry.bat {
            81..=100 => " ",
            60..=80 => " ",
            40..=59 => " ",
            20..=39 => " ",
            0..=19 => " ",
            _ => " ",
        };
        format!("{} [{} {}]", last, entry.btype, batperc)
    });
    WaybarStatus {
        text,
        class: "default".to_string(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut devices = get_devices().await?;
    devices.sort_by_key(|item| item.name.to_string());
    let status = format_waybar(&devices);
    let json = serde_json::to_string(&status).unwrap();
    println!("{}", json);

    Ok(())
}
