# blackshark-ctl

Rust userspace tool + kernel driver for the Razer BlackShark V3 Pro headset.

## Goals

Two-layer architecture:

**Kernel driver (`hid-blackshark`)**
- Battery level → exposed via UPower / `power_supply` subsystem (shows in GNOME battery indicator automatically)
- Mic mute state → surfaced to the system
- Charging status

**`blackshark-ctl` (userspace)**
- Sidetone level
- EQ profiles
- Mic monitoring
- Anything stateful or complex that doesn't map cleanly to sysfs

## Device Info

- USB VID: `0x1532` (Razer)
- Protocol: 90-byte HID reports
- Interfaced via `hidapi` crate in userspace

## Protocol Status

Partial protocol knowledge already reverse engineered via `usbmon`. Full capture still needed to map:
- Sidetone commands
- EQ commands
- Battery polling request/response
- Mic mute state

## Next Step: Full Protocol Capture

Boot to bare metal Windows, run Razer Synapse + Wireshark with USBPcap. Systematically trigger each feature in isolation:
1. Slide sidetone up/down
2. Toggle mic monitoring
3. Change EQ presets one at a time
4. Let it idle to capture battery polling

Save capture, bring back to Linux, diff the 90-byte reports to identify command bytes for each feature.

## Architecture Notes

- Kernel driver handles system-level surface only (battery, mute state)
- Userspace handles rich config (EQ curves, profiles, sidetone) — doesn't map well to sysfs
- Alternatively, `blackshark-ctl` can expose battery via the UPower D-Bus interface directly (without a kernel driver) — same GNOME integration, less work
- HID subsystem is simpler than DRM; kernel review process is familiar from prior panel driver / MIPI DSI work

## References

- `hid-pulsar` kernel patch — good reference for structure of a minimal HID driver
- OpenRazer — not useful here, focused on RGB lighting, minimal headset support
- UPower D-Bus interface — alternative to kernel `power_supply` for battery reporting from userspace
