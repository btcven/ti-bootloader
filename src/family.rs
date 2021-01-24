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

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

/// The type of the bootloader.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Family {
    /// CC2538 microcontrollers.
    CC2538,
    /// CC26x0 and CC13x0 microcontrollers.
    CC26X0,
    /// CC26x2 and CC13x2 microcontrollers.
    CC26X2,
}

impl Family {
    /// Whether the device supports `COMMAND_RUN`.
    ///
    /// - **Note:** supported only on [`Family::CC2538`].
    #[inline]
    pub fn supports_run(&self) -> bool {
        matches!(*self, Family::CC2538)
    }
    /// Whether the device supports `COMMAND_ERASE`.
    ///
    /// - **Note:** supported only on [`Family::CC2538`].
    #[inline]
    pub fn supports_erase(&self) -> bool {
        matches!(*self, Family::CC2538)
    }

    /// Whether the device supports `COMMAND_SECTOR_ERASE`.
    ///
    /// - **Note:** supported only on [`Family::CC26X0`] and [`Family::CC26X2`].
    #[inline]
    pub fn supports_sector_erase(&self) -> bool {
        matches!(*self, Family::CC26X0 | Family::CC26X2)
    }

    /// Whether the device supports `COMMAND_SET_XOSC`.
    ///
    /// - **Note:** supported only on [`Family::CC2538`].
    #[inline]
    pub fn supports_set_xosc(&self) -> bool {
        matches!(*self, Family::CC2538)
    }

    /// Whether the device supports `COMMAND_BANK_ERASE`.
    ///
    /// - **Note:** supported only on [`Family::CC26X0`] and [`Family::CC26X2`].
    #[inline]
    pub fn supports_bank_erase(&self) -> bool {
        matches!(*self, Family::CC26X0 | Family::CC26X2)
    }

    /// Whether the device supports `COMMAND_SET_CCFG`.
    ///
    /// - **Note:** supported only on [`Family::CC26X0`] and [`Family::CC26X2`].
    #[inline]
    pub fn supports_set_ccfg(&self) -> bool {
        matches!(*self, Family::CC26X0 | Family::CC26X2)
    }

    /// Whether the device supports `COMMAND_DOWNLOAD_CRC`.
    ///
    /// - **Note:** supported only on [`Family::CC26X2`].
    #[inline]
    pub fn supports_download_crc(&self) -> bool {
        matches!(*self, Family::CC26X2)
    }

    /// Sector erase size, in bytes.
    #[inline]
    pub fn sector_size(&self) -> u32 {
        match *self {
            Family::CC2538 => 2048,
            Family::CC26X0 => 4092,
            Family::CC26X2 => 8192,
        }
    }

    /// Flash base size.
    #[inline]
    pub fn flash_base(&self) -> u32 {
        match *self {
            Family::CC2538 => 0x00200000,
            Family::CC26X0 | Family::CC26X2 => 0x00000000,
        }
    }

    /// Convert a flash address to the flash page.
    #[inline]
    pub fn address_to_page(&self, address: u32) -> u32 {
        (address - self.flash_base()) / self.sector_size()
    }
}

#[derive(Debug)]
pub struct ParseFamilyError;

impl Display for ParseFamilyError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        write!(fmt, "invalid value, family must be one of: `cc2538`, `cc26x0` or `cc26x2`")
    }
}

impl Error for ParseFamilyError {}

impl FromStr for Family {
    type Err = ParseFamilyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cc2538" | "CC2538" => Ok(Family::CC2538),
            "cc26x0" | "CC26X0" => Ok(Family::CC26X0),
            "cc26x2" | "CC26X2" => Ok(Family::CC26X2),
            _ => Err(ParseFamilyError),
        }
    }
}
