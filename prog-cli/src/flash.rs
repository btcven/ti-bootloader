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

use std::{fs::File, io::Read, path::PathBuf};

use serial::SystemPort;
use ti_sbl::{
    util::{Transfer, CCFG_SIZE},
    Device, Family,
};

use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use indicatif::{ProgressBar, ProgressStyle};

/// Flash subcommand entry point.
pub fn flash(
    args: &ArgMatches<'_>,
    flash_size: u32,
    device: &mut Device<SystemPort>,
) -> Result<()> {
    let args = FlashArgs::from_matches(args)?;

    let mut binary_file = File::open(&args.binary_path).with_context(|| {
        format!(
            "Couldn't open firmware file: `{}`",
            args.binary_path.display()
        )
    })?;

    let mut binary = Vec::new();
    binary_file
        .read_to_end(&mut binary)
        .context("Failed to read firmware file contents")?;

    if binary.len() > flash_size as usize {
        bail!("Binary size is too large");
    }

    log::info!(
        "Binary file: `{}`",
        args.binary_path.file_name().unwrap().to_string_lossy()
    );
    log::info!("Binary file size: {} bytes", binary.len());

    let family = device.family();

    if args.address < family.flash_base() {
        bail!(
            "Start address out of range (base is: {:#X})",
            family.flash_base()
        );
    }

    let overwrites_ccfg = may_overwrite_ccfg(flash_size, args.address, &binary);

    if matches!(family, Family::CC26X0 | Family::CC26X2)
        && overwrites_ccfg
        && !args.force
    {
        bail!("Binary may overwrite the CCFG, use --force if you want to flash it anyway");
    }

    if args.write_erase {
        log::info!(
            "{} bytes will be erased at start address {}",
            binary.len(),
            args.address
        );

        let len = if overwrites_ccfg {
            binary.len() - CCFG_SIZE
        } else {
            binary.len()
        };

        let progress_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");

        let progress_bar = ProgressBar::new(100);
        progress_bar.set_style(progress_style);
        progress_bar.set_message("Erasing sectors");

        ti_sbl::util::erase_flash_range(
            device,
            args.address,
            len as u32,
            |progress, addr| {
                progress_bar.set_message(&format!(
                    "{:.1} - Erasing sector {:#X}",
                    progress, addr
                ));
                progress_bar.inc(1);
            },
        )
        .context("Couldn't erase flash")?;

        progress_bar.finish_with_message("Sectors erased");
    }

    let end_addr = args.address + binary.len() as u32;
    if end_addr > family.flash_base() + flash_size {
        bail!("Binary file is too large for flash (end address: {:#X}, flash size: {:#X})",
              end_addr, flash_size);
    }

    // CCFG is sent separately, and doesn't
    // expect an ACK in return, if the device locks itself.
    let transfers = if matches!(family, Family::CC26X0 | Family::CC26X2)
        && overwrites_ccfg
    {
        debug_assert!(args.force);

        let mut txs = Vec::with_capacity(2);

        txs.push(Transfer {
            data: &binary[..binary.len() - CCFG_SIZE],
            start_address: args.address,
            expect_ack: true,
        });

        txs.push(Transfer {
            data: &binary[binary.len() - CCFG_SIZE..],
            start_address: (args.address + binary.len() as u32)
                - CCFG_SIZE as u32,
            expect_ack: false,
        });

        txs
    } else {
        vec![Transfer {
            data: &binary,
            start_address: args.address,
            expect_ack: true,
        }]
    };
    let progress_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");

    let progress_bar = ProgressBar::new(100);
    progress_bar.set_style(progress_style);
    progress_bar.set_message("Writing flash");

    ti_sbl::util::write_flash_range(
        device,
        &transfers,
        |txfer, progress, chunk_index, chunk_addr| {
            progress_bar.set_message(&format!(
                "{:.1} Writing flash, transfer #{}, chunk #{} ({:#X})",
                progress, txfer, chunk_index, chunk_addr
            ));
            progress_bar.inc(1);
        },
    )
    .context("Couldn't flash binary")?;
    progress_bar.finish_with_message("Transfers finished");

    Ok(())
}

struct FlashArgs {
    binary_path: PathBuf,
    address: u32,
    write_erase: bool,
    force: bool,
}

impl FlashArgs {
    pub fn from_matches(args: &ArgMatches<'_>) -> Result<FlashArgs> {
        Ok(FlashArgs {
            binary_path: args.value_of("BIN").unwrap().parse().context("Invalid binary file path")?,
            address: u32::from_str_radix(&args.value_of("address").map(|a| {
                let mut a = a.to_string();
                if a.starts_with("0x") {
                    a.split_off(2)
                } else {
                    a
                }
            }).unwrap(), 16).context("Invalid flash address, must be an hexadecimal number, e.g.: 0x00000000")?,
            write_erase: args.is_present("write-erase"),
            force: args.is_present("force"),
        })
    }
}

fn may_overwrite_ccfg(
    flash_size: u32,
    binary_offset_in_flash: u32,
    binary: &[u8],
) -> bool {
    let ccfg_offset = flash_size - CCFG_SIZE as u32;
    log::trace!("CCFG offset: {:X}", ccfg_offset);

    let binary_end_addr = binary_offset_in_flash + binary.len() as u32;

    binary_end_addr >= ccfg_offset
}
