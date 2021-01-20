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
use std::{
    io::{self, Write},
    path::PathBuf,
    time::Duration,
};

use serial::SerialPort;

use anyhow::{Context, Error, Result};
use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};

mod flash;

#[cfg(unix)]
const DEFAULT_PORT: &str = "/dev/ttyACM0";
#[cfg(windows)]
const DEFAULT_PORT: &str = "COM0";

fn main() -> Result<()> {
    let app = App::new("TI Serial Interface Bootloader Programmer")
        .setting(AppSettings::ColoredHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Programmer for Texas Instruments Serial Interface Bootloader\nProject website: https://locha.io/software/ti-sbl")
        .arg(
            Arg::with_name("PORT")
                .short("p")
                .long("port")
                .default_value(DEFAULT_PORT)
                .required(true)
                .help("Serial port to use")
        )
        .arg(
            Arg::with_name("family")
                .long("family")
                .default_value("cc26x2")
                .required(true)
                .help("Device family [cc2538|cc26x0|cc26x2]")
        )
        .arg(
            Arg::with_name("baudrate")
                .short("b")
                .long("baudrate")
                .default_value("500000")
                .required(true)
                .help("Serial port baud rate [110|300|600|1200|2400|4800|9600|19200|38400|57600|115200|other]")
        )
        .arg(
            Arg::with_name("enable-xosc")
                .short("x")
                .long("enable-xosc")
                .help("Switch to XOSC (supported only on CC2538 devices)")
        )
        .arg(
            Arg::with_name("bootloader-invoke")
                .long("bl-invoke")
                .help("Invoke the bootloader by toggling the DTR/CTS pins (on supported boards). By default DTR is connected to the bootloader pin (the pin set on CCA/CCFG) and RTS is connected to !RESET. To invert this, use --bl-inverted. The default level of the bootloader pin is active high, to use an active low polarity use --bl-active-low")
        )
        .arg(
            Arg::with_name("bootloader-inverted")
                .long("bl-inverted")
                .help("Invert the pins when using --bl-invoke flag, this sets DTR to !RESET and RTS to bootloader pin")
        )
        .arg(
            Arg::with_name("bootloader-active-low")
                .long("bl-active-low")
                .help("Use an active-low level when using --bl-invoke flag, this sets level of the bootloader pin to active-low")
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity, -v (debug), -vv (trace)")
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
                    Arg::with_name("address")
                        .short("a")
                        .long("address")
                        .required(true)
                        .default_value("0x00000000")
                        .help("Address in memory where the binary contents will be flashed")
                )
                .arg(
                    Arg::with_name("write-erase")
                        .short("e")
                        .long("write-erase")
                        .help("Erase first before writing the binary contents, this will erase up to N bytes where N is the binary file size at the specified --address")
                )
                .arg(
                    Arg::with_name("force")
                        .short("f")
                        .long("force")
                        .help("Force the write of the CCFG. Note: depending on the values of the binary file, you may end up locking yourself out of the device.")
                )
        );

    // When double clicking the binary the binary will be paused. Useful on
    // windows, since the Console window will be closed inmediately.
    #[cfg(windows)]
    let app = app.setting(AppSettings::WaitOnError);

    let matches = app.get_matches();

    init_logger(match matches.occurrences_of("v") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2..=u64::MAX => log::LevelFilter::Trace,
    })?;

    // Sanity checks first
    if matches.is_present("bootloader-inverted")
        && !matches.is_present("bootloader-invoke")
    {
        anyhow::bail!("--bl-inverted can't be used if --bl-invoke is not specified. See --help for more information");
    }

    if matches.is_present("bootloader-active-low")
        && !matches.is_present("bootloader-invoke")
    {
        anyhow::bail!("--bl-active-low can't be used if --bl-invoke is not specified. See --help for more information");
    }

    let opts = Opts {
        #[cfg(unix)]
        port: matches.value_of("PORT").unwrap().parse()?,
        #[cfg(windows)]
        port: matches.value_of("PORT").unwrap().map(OsString::from),
        family: family_from_str(matches.value_of("family").unwrap())?,
        baudrate: matches.value_of("baudrate").unwrap().parse::<usize>().map(
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
        enable_xosc: matches.is_present("enable-xosc"),
        bootloader_invoke: matches.is_present("bootloader-invoke"),
        bootloader_inverted: matches.is_present("bootloader-inverted"),
        bootloader_active_low: matches.is_present("bootloader-active-low"),
    };

    if opts.enable_xosc && !opts.family.supports_set_xosc() {
        anyhow::bail!("XOSC can only be enabled on CC2538 family");
    }

    log::info!("Opening serial port `{}`", opts.port_to_string());
    log::info!("Baudrate: {}", baudrate_to_usize(opts.baudrate));
    let mut port = serial::SystemPort::open(&opts.port).with_context(|| {
        format!("Couldn't open serial port `{}`", opts.port_to_string())
    })?;

    let mut settings = ti_sbl::port_settings();
    settings.baud_rate = opts.baudrate;

    port.set_timeout(Duration::from_millis(200))?;
    port.configure(&settings)?;

    if opts.bootloader_invoke {
        log::info!("Invoking bootloader");
        ti_sbl::invoke_bootloader(
            &mut port,
            opts.bootloader_inverted,
            !opts.bootloader_active_low,
        )
        .context("Failed to invoke bootloader")?;
    }

    log::info!("Initializing communications with the device");
    let mut device = ti_sbl::Device::new(port, opts.family)
        .context("Failed to synchronize with the bootloader")?;

    log::info!("Pinging device");
    if !device.ping()? {
        anyhow::bail!("Ping command wasn't acknowledged");
    }

    if opts.enable_xosc {
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
        log::info!("IEEE 802.15.4g secondary address: {}", format_addr(secondary));
    }

    match matches.subcommand() {
        ("flash", Some(m)) => flash::flash(m, flash_size, &mut device)?,
        _ => {
            println!("Error: Sub-command required");
            println!("{}", matches.usage());
        }
    }

    Ok(())
}

struct Opts {
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

impl Opts {
    pub fn port_to_string(&self) -> String {
        #[cfg(unix)]
        let port = self.port.display().to_string();
        #[cfg(windows)]
        let port = self.port.to_string_lossy();

        port
    }
}

fn family_from_str(s: &str) -> Result<ti_sbl::Family> {
    match s {
        "cc2538" => Ok(ti_sbl::Family::CC2538),
        "cc26x0" => Ok(ti_sbl::Family::CC26X0),
        "cc26x2" => Ok(ti_sbl::Family::CC26X2),
        _ => Err(Error::msg(format!(
            "invalid family, must be one of: `cc2538`, `cc26x0`, `cc26x2`."
        ))),
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

fn init_logger(level: log::LevelFilter) -> Result<()> {
    let mut logger = env_logger::Builder::from_env("TI_SBL_LOG");
    logger.filter_level(level);

    #[cfg(unix)]
    logger.format(log_format_color);
    #[cfg(not(unix))]
    logger.format(log_format_no_color);

    logger.try_init().context("Failed to initialize logger")
}

#[cfg(unix)]
fn log_format_color(
    fmt: &mut env_logger::fmt::Formatter,
    record: &log::Record<'_>,
) -> io::Result<()> {
    let level = match record.level() {
        log::Level::Error => ansi_term::Color::Red.bold().paint("ERROR"),
        log::Level::Warn => ansi_term::Color::Yellow.bold().paint("WARN"),
        log::Level::Info => ansi_term::Color::Green.bold().paint("INFO"),
        log::Level::Debug => ansi_term::Color::Cyan.bold().paint("DBG"),
        log::Level::Trace => ansi_term::Color::Cyan.bold().paint("TRACE"),
    };

    writeln!(fmt, "[{}] - {}", level, record.args())
}

#[cfg(not(unix))]
fn log_format_no_color(
    fmt: &mut env_logger::fmt::Formatter,
    record: &log::Record<'_>,
) -> io::Result<()> {
    let level = match record.level() {
        log::Level::Error => "ERROR",
        log::Level::Warn => "WARN",
        log::Level::Info => "INFO",
        log::Level::Debug => "DBG",
        log::Level::Trace => "TRACE",
    };

    writeln!(fmt, "[{}] - {}", level, record.args())
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
