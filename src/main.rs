mod battery;
mod device;
mod protocol;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "blackshark-ctl", about = "Control the Razer BlackShark V3 Pro headset")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Set sidetone level (0–100)
    Sidetone {
        #[arg(value_name = "LEVEL", value_parser = clap::value_parser!(u8).range(0..=100))]
        level: u8,
    },
    /// Query battery level
    Battery,
    /// Set mic monitoring level (0–100)
    Monitor {
        #[arg(value_name = "LEVEL", value_parser = clap::value_parser!(u8).range(0..=100))]
        level: u8,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Sidetone { level } => {
            let dev = device::open()?;
            cmd_sidetone(&dev, level)
        }
        Command::Battery => {
            let dev = device::open()?;
            cmd_battery(&dev)
        }
        Command::Monitor { level } => {
            let dev = device::open()?;
            cmd_monitor(&dev, level)
        }
    }
}

fn cmd_sidetone(dev: &hidapi::HidDevice, level: u8) -> Result<()> {
    use protocol::{cmd, Report};
    let report = Report::new(0x1f, cmd::SIDETONE_CLASS, cmd::SIDETONE_ID, &[level]);
    device::send(dev, &report)?;
    println!("sidetone set to {level}");
    Ok(())
}

fn cmd_battery(dev: &hidapi::HidDevice) -> Result<()> {
    let state = battery::query(dev)?;
    let charging = if state.charging { " (charging)" } else { "" };
    println!("battery: {:.0}%{charging}", state.percentage);
    Ok(())
}

fn cmd_monitor(dev: &hidapi::HidDevice, level: u8) -> Result<()> {
    use protocol::{cmd, Report};
    let report = Report::new(0x1f, cmd::MIC_MONITOR_CLASS, cmd::MIC_MONITOR_ID, &[level]);
    device::send(dev, &report)?;
    println!("mic monitoring set to {level}");
    Ok(())
}
