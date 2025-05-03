use core::time;
use std::{ffi::OsStr, thread};

use futures_util::stream::StreamExt;
use zbus::{proxy, zvariant::OwnedObjectPath, Connection};

#[proxy(
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1",
    interface = "org.freedesktop.systemd1.Manager"
)]
trait Systemd1Manager {
    // Defines signature for D-Bus signal named `JobNew`
    #[zbus(signal)]
    fn unit_new(&self, string: String, path: OwnedObjectPath) -> zbus::Result<()>;
    #[zbus(signal)]
    fn unit_removed(&self, string: String, path: OwnedObjectPath) -> zbus::Result<()>;
}

fn notify_waybar() {
    thread::sleep(time::Duration::from_millis(2000));
    let mut system = sysinfo::System::new();
    system.refresh_all();

    let pid = system
        .processes_by_exact_name(OsStr::new("waybar"))
        .next()
        .unwrap()
        .pid()
        .as_u32() as i32;

    let signal_number = { libc::SIGRTMIN() + 13 };
    let _ = unsafe { libc::kill(pid, signal_number) };
}

async fn watch_systemd_jobs() -> anyhow::Result<()> {
    let connection = Connection::system().await?;

    let systemdproxy = Systemd1ManagerProxy::new(&connection).await?;

    let mut new_devices_stream = systemdproxy.receive_unit_new().await?;
    let mut devies_removed_stream = systemdproxy.receive_unit_removed().await?;

    futures_util::try_join!(
        async {
            while let Some(msg) = devies_removed_stream.next().await {
                let args: UnitRemovedArgs = msg.args().expect("Error parsing message");
                if args.string.contains("bluetooth") && args.string.contains("sys-subsystem") {
                    println!("Bluetooth Device removed");
                    notify_waybar();
                }
            }
            Ok::<(), zbus::Error>(())
        },
        async {
            while let Some(msg) = new_devices_stream.next().await {
                let args: UnitNewArgs = msg.args().expect("Error parsing message");
                if args.string.contains("bluetooth") && args.string.contains("sys-subsystem") {
                    println!("Bluetooth Device added");
                    notify_waybar();
                }
            }
            Ok(())
        }
    )?;

    panic!("Stream ended unexpectedly");
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    watch_systemd_jobs().await
}
