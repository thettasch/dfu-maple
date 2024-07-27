use anyhow::{Context, Result};
use dfu_core::DfuIo;
use dfu_libusb::*;

#[derive(clap::Parser)]
pub struct Cli {
    /// Specify Vendor/Product ID(s) of DFU device.
    #[clap(
        long,
        short,
        parse(try_from_str = Self::parse_vid_pid), name = "vendor>:<product",
        default_value = "1EAF:0003",
    )]
    device: (u16, u16),

    /// Specify the DFU Interface number.
    #[clap(long, short, default_value = "0")]
    intf: u8,

    
    /// Reset serial port
    #[clap(short, long)]
    serial_port: Option<String>,


    /// Specify the Altsetting of the DFU Interface by number.
    #[clap(long, short, default_value = "0")]
    alt: u8,

    /// Enable verbose logs.
    #[clap(long, short)]
    verbose: bool,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        let Cli {
            device,
            intf,
            alt,
            serial_port,
            verbose,
        } = self;
        let log_level = if verbose {
            simplelog::LevelFilter::Trace
        } else {
            simplelog::LevelFilter::Info
        };
        simplelog::SimpleLogger::init(log_level, Default::default())?;

        if let Some(serial_port) = &serial_port {
            // println!("Reseting MCU at {serial_port}");
            let bar = indicatif::ProgressBar::new_spinner();
            bar.set_message(format!("Reseting MCU at {serial_port}"));
            bar.tick();
            match reset_mcu(&serial_port) {
                Ok(()) => {
                    for _ in 0..3 {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        bar.tick();
                    }
                }
                Err(e) => {
                    bar.set_message(format!("Failed to reset MCU at {serial_port}: {e}"));
                }
            }
            bar.finish();
        }

        let (vid, pid) = device;
        let context = rusb::Context::new()?;

        let device: Dfu<rusb::Context> =
            DfuLibusb::open(&context, vid, pid, intf, alt).context("could not open device")?;

        println!("{:?}", device.into_inner().functional_descriptor());

        Ok(())
    }

    pub fn parse_vid_pid(s: &str) -> Result<(u16, u16)> {
        let (vid, pid) = s
            .split_once(':')
            .context("could not parse VID/PID (missing `:')")?;
        let vid = u16::from_str_radix(vid, 16).context("could not parse VID")?;
        let pid = u16::from_str_radix(pid, 16).context("could not parse PID")?;

        Ok((vid, pid))
    }
}

fn main() -> Result<()> {
    <Cli as clap::Parser>::from_args().run()
}
