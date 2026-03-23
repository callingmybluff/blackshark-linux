/// Battery reporting via the UPower D-Bus interface.
///
/// We expose ourselves as a UPower device so GNOME (and anything else that
/// speaks UPower) picks up the headset battery level automatically — same
/// integration as a kernel power_supply driver, but from userspace.
///
/// UPower interface: org.freedesktop.UPower.Device
/// Well-known path:  /org/freedesktop/UPower/devices/headset_blackshark_v3_pro
///
/// TODO: implement after the HID battery polling command is confirmed from pcap.

use anyhow::Result;

/// Battery state as reported to UPower.
#[derive(Debug, Clone, Copy)]
pub struct BatteryState {
    /// Percentage, 0.0–100.0.
    pub percentage: f64,
    /// Whether the device is currently on charge.
    pub charging: bool,
}

/// Query the headset for its current battery state.
///
/// Stubbed out — the HID command bytes are not yet known.
#[allow(unused_variables)]
pub fn query(dev: &hidapi::HidDevice) -> Result<BatteryState> {
    // TODO: once pcap reveals the battery request/response format:
    //   1. Build a Report with cmd::BATTERY_CLASS / cmd::BATTERY_ID
    //   2. Call device::send()
    //   3. Parse percentage + charging flag from response.data()
    anyhow::bail!("battery command not yet implemented (pending Windows pcap)")
}

/// Publish battery state over D-Bus so UPower / GNOME can display it.
///
/// TODO: implement the UPower Device interface with zbus.
pub async fn publish_upower(_state: BatteryState) -> Result<()> {
    anyhow::bail!("UPower D-Bus publishing not yet implemented")
}
