// Copyright 2021 Locha Mesh Developers <contact@locha.io>
//
// Based on the previous work of cc2538-bsl and Texas Instruments sblAppEx
// 1.03.00.00 (swra466c.zip).
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

//! # Utilities
//!
//! These are convenience functions to read common parameters of the devices,
//! such as the IEEE 802.15.5g address, BLE MAC address, the flash size in
//! bytes, etc.

use std::{convert::TryInto, io};

use crate::{
    Device, Family, COMMAND_RET_FLASH_FAIL, COMMAND_RET_INVALID_ADR,
    COMMAND_RET_INVALID_CMD, COMMAND_RET_SUCCESS, COMMAND_RET_UNKNOWN_CMD,
    MAX_BYTES_PER_TRANSFER,
};

/// CC26xx/CC13xx CCFG size in bytes.
pub const CCFG_SIZE: usize = 88;
/// The value of an invalid IEEE/BLE address in the CCFG.
pub const INVALID_ADDR: [u8; 8] =
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

const REG32_SIZE: usize = 4;
/// FLASH.FLASH_SIZE register on CC13xx/CC26xx
const CC26XX_FLASH_O_FLASH_SIZE: u32 = 0x4003002C;
const CC26XX_FCFG1_O_MAC_15_4_0: u32 = 0x000002F0;
/// FLASH_CTRL.DIECFG0 register on CC2538
const CC2538_FLASH_CTRL_O_DIECFG0: u32 = 0x400D3014;

/// Erase a flash range.
pub fn erase_flash_range<P>(
    device: &mut Device<P>,
    start_address: u32,
    byte_count: u32,
) -> io::Result<()>
where
    P: serial::SerialPort,
{
    let family = device.family();
    if family.supports_erase() {
        device.erase(start_address, byte_count)?;
    } else if family.supports_sector_erase() {
        let sector_size = family.sector_size();
        let sector_count = if (byte_count % sector_size) != 0 {
            (byte_count / sector_size) + 1
        } else {
            byte_count / sector_size
        };

        for i in 0..sector_count {
            let sector_address = start_address + (i * sector_size);
            log::info!("Erasing sector #{}, address: {:#X}", i, sector_address);

            device.sector_erase(sector_address)?;

            let ret = device.get_status()?;
            if ret != COMMAND_RET_SUCCESS {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "CMD_SECTOR_ERASE failed: `{}` ({:#X})",
                        status_code_to_str(ret),
                        ret
                    ),
                ));
            }
        }
    } else {
        unreachable!();
    }

    Ok(())
}

#[derive(Debug)]
pub struct Transfer<'a> {
    pub data: &'a [u8],
    pub start_address: u32,
    pub expect_ack: bool,
}

/// Write the flash
pub fn write_flash_range<'a, P>(
    device: &mut Device<P>,
    transfers: &[Transfer<'a>],
) -> io::Result<()>
where
    P: serial::SerialPort,
{
    let family = device.family();

    log::info!("{} transfers", transfers.len());

    for (txfer_index, transfer) in transfers.iter().enumerate() {
        let chunks = transfer.data.len() / MAX_BYTES_PER_TRANSFER;
        log::info!("Chunks for transfer #{}: {}", txfer_index, chunks);

        // Prepare device for flash download.
        device.download(
            transfer.start_address,
            transfer.data.len().try_into().unwrap(),
        )?;

        // Each download command requires to check the latest
        // status to verify it worked.
        let ret = device.get_status()?;

        if ret != COMMAND_RET_SUCCESS {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "CMD_DOWNLOAD failed: `{}` ({:#X})",
                    status_code_to_str(ret),
                    ret
                ),
            ));
        }

        let mut bytes_left = transfer.data.len();
        let mut data_offset = 0;
        let mut chunk_index = 0;

        while bytes_left > 0 {
            let bytes_in_transfer = MAX_BYTES_PER_TRANSFER.min(bytes_left);
            let chunk = &transfer.data[data_offset..];
            let chunk = &chunk[..bytes_in_transfer];

            let chunk_addr = transfer.start_address + data_offset as u32;
            log::info!(
                "Writing chunk #{} ({} B) at address {:#X}",
                chunk_index,
                chunk.len(),
                chunk_addr
            );

            let ack = device.send_data(&chunk)?;
            if transfer.expect_ack {
                if !ack {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "Chunk #{} of size {} not acknowledged at address {:#X} (page: {}) at transfer #{}",
                            chunk_index, chunk.len(), chunk_addr, family.address_to_page(chunk_addr),
                            txfer_index,
                        )
                    ));
                }

                let ret = device.get_status()?;
                if ret != COMMAND_RET_SUCCESS {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "CMD_SEND_DATA failed: `{}` ({:#X})",
                            status_code_to_str(ret),
                            ret
                        ),
                    ));
                }
            }

            bytes_left -= bytes_in_transfer;
            data_offset += bytes_in_transfer;
            chunk_index += 1;
        }
    }

    Ok(())
}

/// Read memory.
pub fn memory_read_32<P>(
    _device: &mut Device<P>,
    _start_address: u32,
    _data: &mut [u8],
) -> io::Result<()>
where
    P: serial::SerialPort,
{
    todo!();
}

/// Reads the flash size from the memory.
pub fn read_flash_size<P>(device: &mut Device<P>) -> io::Result<u32>
where
    P: serial::SerialPort,
{
    let addr = match device.family() {
        Family::CC2538 => CC2538_FLASH_CTRL_O_DIECFG0,
        Family::CC26X0 | Family::CC26X2 => CC26XX_FLASH_O_FLASH_SIZE,
    };

    let mut reg = [0u8; REG32_SIZE];
    device.memory_read_32(addr, &mut reg)?;

    match device.family() {
        Family::CC2538 => {
            let flash_ctrl = u32::from_le_bytes(reg);
            let flash_size = (flash_ctrl >> 4) & 0x07;
            match flash_size {
                1 => Ok(0x20000), // 128 KB
                2 => Ok(0x40000), // 256 KB
                3 => Ok(0x60000), // 384 KB
                4 => Ok(0x80000), // 512 KB
                0 => Ok(0x10000), //  64 KB
                _ => Ok(0x10000), // All invalid values are interpreted as 64 KB
            }
        }
        Family::CC26X0 | Family::CC26X2 => {
            let mut flash_size = u32::from_le_bytes(reg);
            flash_size &= 0xFF;

            Ok(flash_size * device.family().sector_size())
        }
    }
}

/// Read IEEE 802.15.4g MAC address.
pub fn read_ieee_address<P>(
    device: &mut Device<P>,
) -> io::Result<([u8; 8], [u8; 8])>
where
    P: serial::SerialPort,
{
    let primary_addr_offset = match device.family() {
        Family::CC2538 => 0x00280028,
        Family::CC26X0 | Family::CC26X2 => CC26XX_FCFG1_O_MAC_15_4_0,
    };

    let secondary_addr_offset = match device.family() {
        Family::CC2538 => 0x0027ffcc,
        Family::CC26X0 | Family::CC26X2 => {
            let ccfg_offset = read_flash_size(device)? - CCFG_SIZE as u32;

            ccfg_offset + 0x20
        }
    };

    let mut primary = [0u8; 8];
    device.memory_read_32(primary_addr_offset, &mut primary)?;

    let mut secondary = [0u8; 8];
    device.memory_read_32(secondary_addr_offset, &mut secondary)?;

    Ok((primary, secondary))
}

pub fn status_code_to_str(ret: u8) -> &'static str {
    match ret {
        COMMAND_RET_SUCCESS => "COMMAND_RET_SUCCESS",
        COMMAND_RET_UNKNOWN_CMD => "COMMAND_RET_UNKNOWN_CMD",
        COMMAND_RET_INVALID_CMD => "COMMAND_RET_INVALID_CMD",
        COMMAND_RET_INVALID_ADR => "COMMAND_RET_INVALID_ADR",
        COMMAND_RET_FLASH_FAIL => "COMMAND_RET_FLASH_FAIL",
        _ => "Unknown",
    }
}
