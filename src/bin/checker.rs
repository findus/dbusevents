use anyhow::Error;
use serde::Deserialize;
use std::collections::HashMap;
use zbus::zvariant::OwnedValue;
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

    let mut vec: Vec<BluetoothStatus> = vec![];
    let mut failcount = 20;

    loop {
        failcount = failcount - 1;
        if failcount <= 0 {
            break Ok(vec);
        }

        let objects: BluezManagedObjects = proxy.call("GetManagedObjects", &()).await?;

        for (_, interfaces) in objects {
            if let Some(device_props) = interfaces.get("org.bluez.Device1") {
                if let Some(connected_val) = device_props.get("Connected") {
                    if let Ok(true) = connected_val.downcast_ref::<bool>() {
                        let Some(Ok(name)) = device_props
                            .get("Name")
                            .and_then(|v| v.downcast_ref::<String>().into())
                        else {
                            continue;
                        };

                        let Some(Ok(address)) = device_props
                            .get("Address")
                            .and_then(|v| v.downcast_ref::<String>().into())
                        else {
                            continue;
                        };

                        let Some(Ok(icon_type)) = device_props
                            .get("Icon")
                            .and_then(|v| v.downcast_ref::<String>().into())
                        else {
                            continue;
                        };

                        let icon = match icon_type.as_str() {
                            "input-mouse" => "".to_string(),
                            "input-keyboard" => "".to_string(),
                            "audio-headset" | "audio-headphones" => "".to_string(),
                            _ => icon_type,
                        };

                        // Try to get battery level if available
                        let battery_level = interfaces
                            .get("org.bluez.Battery1")
                            .and_then(|battery_props| {
                                battery_props
                                    .get("Percentage")
                                    .and_then(|v| v.downcast_ref::<u8>().into())
                            })
                            .unwrap_or_else(|| Ok(0));

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
    }
}

#[derive(serde::Serialize, Deserialize, Debug)]
pub struct WaybarStatus {
    text: String,
    class: String,
}

fn format_waybar(devices: &[BluetoothStatus]) -> Option<WaybarStatus> {
    if devices.is_empty() {
        return Option::None;
    }
    let text = devices.iter().fold("".to_string(), |last, entry| {
        let batperc = match entry.bat {
            81..=100 => "",
            60..=80 => "",
            40..=59 => "",
            20..=39 => "",
            0..=19 => "",
            _ => "",
        };
        format!("{} [{} {}]", last, entry.btype, batperc)
    });
    Some(WaybarStatus {
        text,
        class: "default".to_string(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut devices = get_devices().await?;
    devices.sort_by_key(|item| item.name.to_string());
    if let Some(status) = format_waybar(&devices) {
        println!("{}", serde_json::to_string(&status).unwrap());
    }
    Ok(())
}
