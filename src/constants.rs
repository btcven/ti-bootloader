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

pub const CMD_PING: u8                  = 0x20;
pub const CMD_DOWNLOAD: u8              = 0x21;
pub const CC2538_CMD_RUN: u8            = 0x22;
pub const CMD_GET_STATUS: u8            = 0x23;
pub const CMD_SEND_DATA: u8             = 0x24;
pub const CMD_RESET: u8                 = 0x25;
pub const CC2538_CMD_ERASE: u8          = 0x26;
pub const CC26X0_CMD_SECTOR_ERASE: u8   = 0x26;
pub const CMD_CRC32: u8                 = 0x27;
pub const CMD_GET_CHIP_ID: u8           = 0x28;
pub const CC2538_CMD_SET_XOSC: u8       = 0x29;
pub const CMD_MEMORY_READ: u8           = 0x2A;
pub const CMD_MEMORY_WRITE: u8          = 0x2B;
pub const CC26X0_CMD_BANK_ERASE: u8     = 0x2C;
pub const CC26X0_CMD_SET_CCFG: u8       = 0x2D;
pub const CC26X2_CMD_DOWNLOAD_CRC: u8   = 0x2F;

/// ACK byte
pub const ACK: u8                       = 0xCC;
/// NACK byte
pub const NACK: u8                      = 0x33;

/// Maximum bytes per transfer, on [`CMD_SEND_DATA`] commands.
pub const MAX_BYTES_PER_TRANSFER: usize = 252;

pub const COMMAND_RET_SUCCESS: u8       = 0x40;
pub const COMMAND_RET_UNKNOWN_CMD: u8   = 0x41;
pub const COMMAND_RET_INVALID_CMD: u8   = 0x42;
pub const COMMAND_RET_INVALID_ADR: u8   = 0x43;
pub const COMMAND_RET_FLASH_FAIL: u8    = 0x44;
