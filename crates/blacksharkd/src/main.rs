mod config;
mod dbus;
mod hid_actor;
mod state;

use std::time::Duration;

use anyhow::Result;
use tokio::sync::{mpsc, watch};
use tracing::{info, warn};
use zbus::ConnectionBuilder;

use config::Config;
use state::SharedState;

const TICK_INTERVAL: Duration = Duration::from_secs(5);
const DEBOUNCE_INTERVAL: Duration = Duration::from_millis(500);
const DBUS_PATH: &str = "/net/blackshark1/Headset";
const DBUS_NAME: &str = "net.blackshark1";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blacksharkd=info".into()),
        )
        .init();

    // Load config from disk (or create defaults).
    let initial_config = match config::load() {
        Ok(c) => {
            info!(path = %config::config_path().unwrap().display(), "loaded config");
            c
        }
        Err(e) => {
            warn!("could not load config, using defaults: {e}");
            Config::default()
        }
    };

    // Config watch channel — D-Bus methods send updated configs here.
    let (config_tx, mut config_rx) = watch::channel(initial_config.clone());

    let (cmd_tx, cmd_rx) = mpsc::channel::<hid_actor::HidCommand>(32);
    let (state_tx, state_rx) = watch::channel(SharedState::default());

    // Spawn HID actor. Pass initial config so it can restore on first connect.
    hid_actor::spawn(cmd_rx, state_tx, initial_config);

    // Periodic tick → drives reconnect attempts and battery polling.
    let tick_tx = cmd_tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(TICK_INTERVAL);
        loop {
            interval.tick().await;
            if tick_tx.send(hid_actor::HidCommand::Tick).await.is_err() {
                break;
            }
        }
    });

    // Debounced config writer + apply-to-device task.
    // Watches for config changes, waits 500ms of quiet, then saves to disk
    // and tells the HID actor to apply the new values.
    let apply_tx = cmd_tx.clone();
    tokio::spawn(async move {
        loop {
            if config_rx.changed().await.is_err() {
                break;
            }
            // Debounce: wait for changes to settle.
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(DEBOUNCE_INTERVAL) => break,
                    res = config_rx.changed() => {
                        if res.is_err() { return; }
                        // changed again — reset the timer
                    }
                }
            }
            let cfg = config_rx.borrow().clone();
            if let Err(e) = config::save(&cfg) {
                warn!("failed to save config: {e}");
            } else {
                info!("config saved");
            }
            let _ = apply_tx.send(hid_actor::HidCommand::ApplyConfig { config: cfg }).await;
        }
    });

    // D-Bus service.
    let iface = dbus::HeadsetInterface::new(cmd_tx, state_rx.clone(), config_tx);

    let conn = ConnectionBuilder::session()?
        .name(DBUS_NAME)?
        .serve_at(DBUS_PATH, iface)?
        .build()
        .await?;

    info!("running on {DBUS_NAME}");

    // Watch for battery changes and emit the BatteryChanged signal.
    let mut watch_rx = state_rx;
    let conn2 = conn.clone();
    tokio::spawn(async move {
        let mut last_pct = 255u8;
        loop {
            if watch_rx.changed().await.is_err() {
                break;
            }
            let state = watch_rx.borrow().clone();
            if state.connected && state.battery_pct != last_pct {
                last_pct = state.battery_pct;
                let iface_ref = conn2
                    .object_server()
                    .interface::<_, dbus::HeadsetInterface>(DBUS_PATH)
                    .await;
                if let Ok(iface_ref) = iface_ref {
                    dbus::HeadsetInterface::battery_changed(
                        iface_ref.signal_context(),
                        state.battery_pct,
                        state.charging,
                    )
                    .await
                    .ok();
                }
            }
        }
    });

    std::future::pending::<()>().await;
    Ok(())
}
