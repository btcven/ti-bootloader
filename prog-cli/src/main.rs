// Copyright 2021 Locha Mesh Developers <contact@locha.io>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(windows)]
use std::ffi::OsString;
use std::{path::PathBuf, time::Duration};

use serial::SerialPort;

use anyhow::{bail, Context, Result};
use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};

mod flash;

#[cfg(unix)]
const DEFAULT_PORT: &str = "/dev/ttyACM0";
#[cfg(windows)]
const DEFAULT_PORT: &str = "COM0";

fn main() -> Result<()> {
    #[cfg(feature = "pretty-env-logger")]
    pretty_env_logger::init_custom_env("TI_SBL_PROG_LOG");
    #[cfg(not(feature = "pretty-env-logger"))]
    env_logger::init_from_env("TI_SBL_PROG_LOG");

    let args = cli().get_matches_safe()?;

    // Sanity checks first
    if args.is_present("bl-inverted") && !args.is_present("bl-invoke") {
        bail!("--bl-inverted can't be used if --bl-invoke is not specified. See --help for more information");
    }

    if args.is_present("bl-active-low") && !args.is_present("bl-invoke") {
        bail!("--bl-active-low can't be used if --bl-invoke is not specified. See --help for more information");
    }

    let global_args = GlobalArgs {
        #[cfg(unix)]
        port: args.value_of("port").unwrap().parse()?,
        #[cfg(windows)]
        port: args.value_of("port").unwrap().map(OsString::from),
        family: args.value_of("family").unwrap().parse()?,
        baudrate: args.value_of("baudrate").unwrap().parse::<usize>().map(
            |v| match v {
                110 => serial::BaudRate::Baud110,
                300 => serial::BaudRate::Baud300,
                600 => serial::BaudRate::Baud600,
                1200 => serial::BaudRate::Baud1200,
                2400 => serial::BaudRate::Baud2400,
                4800 => serial::BaudRate::Baud4800,
                9600 => serial::BaudRate::Baud9600,
                19200 => serial::BaudRate::Baud19200,
                38400 => serial::BaudRate::Baud38400,
                57600 => serial::BaudRate::Baud57600,
                115200 => serial::BaudRate::Baud115200,
                n => serial::BaudRate::BaudOther(n),
            },
        )?,
        enable_xosc: args.is_present("enable-xosc"),
        bootloader_invoke: args.is_present("bl-invoke"),
        bootloader_inverted: args.is_present("bl-inverted"),
        bootloader_active_low: args.is_present("bl-active-low"),
    };

    if global_args.enable_xosc && !global_args.family.supports_set_xosc() {
        anyhow::bail!("XOSC can only be enabled on CC2538 family");
    }

    log::info!("Opening serial port `{}`", global_args.port_to_string());
    log::info!("Baudrate: {}", baudrate_to_usize(global_args.baudrate));
    let mut port =
        serial::SystemPort::open(&global_args.port).with_context(|| {
            format!(
                "Couldn't open serial port `{}`",
                global_args.port_to_string()
            )
        })?;

    let mut settings = ti_sbl::port_settings();
    settings.baud_rate = global_args.baudrate;

    port.set_timeout(Duration::from_millis(200))?;
    port.configure(&settings)?;

    if global_args.bootloader_invoke {
        log::info!("Invoking bootloader");
        ti_sbl::invoke_bootloader(
            &mut port,
            global_args.bootloader_inverted,
            !global_args.bootloader_active_low,
        )
        .context("Failed to invoke bootloader")?;
    }

    log::info!("Initializing communications with the device");
    let mut device = ti_sbl::Device::new(port, global_args.family)
        .context("Failed to synchronize with the bootloader")?;

    log::info!("Pinging device");
    if !device.ping()? {
        anyhow::bail!("Ping command wasn't acknowledged");
    }

    if global_args.enable_xosc {
        device.set_xosc().context("Couldn't switch to XOSC")?;
        todo!();
    }

    let flash_size = ti_sbl::util::read_flash_size(&mut device)
        .context("Couldn't read flash size")?;
    log::info!("Flash size: {} K", flash_size / 1024);

    let chip_id = device.get_chip_id().context("Couldn't read chip ID")?;
    log::info!("Chip ID: {:#X}", chip_id);

    let (primary, secondary) = ti_sbl::util::read_ieee_address(&mut device)
        .context("Couldn't read IEEE 802.15.4 address")?;
    log::info!("IEEE 802.15.4g primary address: {}", format_addr(primary));
    if secondary != ti_sbl::util::INVALID_ADDR {
        log::info!(
            "IEEE 802.15.4g secondary address: {}",
            format_addr(secondary)
        );
    }

    match args.subcommand() {
        ("flash", Some(m)) => flash::flash(m, flash_size, &mut device)?,
        _ => {
            println!("Error: Sub-command required");
            println!("{}", args.usage());
        }
    }

    Ok(())
}

struct GlobalArgs {
    #[cfg(unix)]
    port: PathBuf,
    #[cfg(windows)]
    port: OsString,
    family: ti_sbl::Family,
    baudrate: serial::BaudRate,
    enable_xosc: bool,
    bootloader_invoke: bool,
    bootloader_inverted: bool,
    bootloader_active_low: bool,
}

impl GlobalArgs {
    pub fn port_to_string(&self) -> String {
        #[cfg(unix)]
        let port = self.port.display().to_string();
        #[cfg(windows)]
        let port = self.port.to_string_lossy();

        port
    }
}

fn baudrate_to_usize(baudrate: serial::BaudRate) -> usize {
    match baudrate {
        serial::BaudRate::Baud110 => 110,
        serial::BaudRate::Baud300 => 300,
        serial::BaudRate::Baud600 => 600,
        serial::BaudRate::Baud1200 => 1200,
        serial::BaudRate::Baud2400 => 2400,
        serial::BaudRate::Baud4800 => 4800,
        serial::BaudRate::Baud9600 => 9600,
        serial::BaudRate::Baud19200 => 19200,
        serial::BaudRate::Baud38400 => 38400,
        serial::BaudRate::Baud57600 => 57600,
        serial::BaudRate::Baud115200 => 115200,
        serial::BaudRate::BaudOther(n) => n,
    }
}

fn format_addr(addr: [u8; 8]) -> String {
    format!(
        "{:X}{:X}:{:X}{:X}:{:X}{:X}:{:X}{:X}:{:X}{:X}:{:X}{:X}:{:X}{:X}:{:X}{:X}",
        addr[0] >> 4, addr[0] & 0x0F,
        addr[1] >> 4, addr[1] & 0x0F,
        addr[2] >> 4, addr[2] & 0x0F,
        addr[3] >> 4, addr[3] & 0x0F,
        addr[4] >> 4, addr[4] & 0x0F,
        addr[5] >> 4, addr[5] & 0x0F,
        addr[6] >> 4, addr[6] & 0x0F,
        addr[7] >> 4, addr[7] & 0x0F,
    )
}

fn cli() -> App<'static, 'static> {
    let app = App::new("TI Serial Interface Bootloader Programmer")
        .usage("ti-sbl-prog [OPTIONS] [SUBCOMMAND] ")
        .setting(AppSettings::ColoredHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Programmer for Texas Instruments Serial Interface Bootloader\nProject homepage: https://github.com/btcven/ti-bootloader")
        .arg(
            opt("port", "Serial port to use")
                .short("p")
                .required(true)
                .default_value(DEFAULT_PORT)
        )
        .arg(
            opt("family", "Family: cc2538, cc26x0, cc26x2")
                .required(true)
                .default_value("cc26x2")
        )
        .arg(
            opt("baudrate", "Serial port baudrate")
                .short("b")
                .required(true)
                .default_value("500000")
        )
        .arg(
            opt("enable-xosc", "Switch to XOSC (only for `cc2538` family)")
                .short("x")
        )
        .arg(
            opt(
                "bl-invoke",
                "Invoke the bootloader by toggling the DTR/CTS pins (on supported boards). By default DTR is connected to the bootloader pin (the pin set on CCA/CCFG) and RTS is connected to !RESET. To invert this, use --bl-inverted. The default level of the bootloader pin is active high, to use an active low polarity use --bl-active-low"
            )
        )
        .arg(
            opt(
                "bl-inverted",
                "Invert the pins when using --bl-invoke flag, this sets DTR to !RESET and RTS to bootloader pin"
            )
        )
        .arg(
            opt(
                "bl-active-low",
                "Use an active-low level when using --bl-invoke flag, this sets level of the bootloader pin to active-low"
            )
        )
        .arg(
            opt("verbose", "Use verbose output: -v (debug), -vv (trace)")
                .short("v")
                .multiple(true)
        )
        .subcommand(
            SubCommand::with_name("flash")
                .about("Flash a binary file")
                .setting(AppSettings::ColoredHelp)
                .arg(
                    Arg::with_name("BIN")
                        .required(true)
                        .takes_value(true)
                        .help("Binary file to flash")
                )
                .arg(
                    opt(
                        "address",
                        "Address in memory where the binary contents will be flashed"
                    )
                        .short("a")
                        .required(true)
                        .default_value("0x00000000")
                )
                .arg(
                    opt(
                        "write-erase",
                        "Erase first before writing the binary contents, this will erase up to N bytes where N is the binary file size at the specified --address"
                    )
                        .short("e")
                )
                .arg(
                    opt(
                        "force",
                        "Force the write of the CCFG. Warning: may lock yourself out of the device."
                    )
                        .short("f")
                )
            );

    // When double clicking the binary the binary will be paused. Useful on
    // windows, since the Console window will be closed inmediately.
    #[cfg(windows)]
    let app = app.setting(AppSettings::WaitOnError);

    app
}

fn opt(name: &'static str, help: &'static str) -> Arg<'static, 'static> {
    Arg::with_name(name).long(name).help(help)
}
