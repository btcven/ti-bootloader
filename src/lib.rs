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

//! # TI Serial Bootloader Interface library
//!
//! This a library to work with the serial interface of the Texas Instruments chips
//! bootloaders.
//!
//! # Tested with the following chips
//!
//! - [CC1312R](https://www.ti.com/product/CC1312R)
//! - [CC1352P](https://www.ti.com/product/CC1352P)
//!
//! # See also
//!
//! - [CC2538/CC26x0/CC26x2 Serial Bootloader Interface](https://www.ti.com/lit/an/swra466c/swra466c.pdf).

use std::{
    cmp::Ordering,
    fmt, io,
    time::{Duration, Instant},
};

use serial::SerialPort;

#[rustfmt::skip]
pub mod constants;
pub mod util;

mod family;
pub use self::family::Family;

/// A TI connected device supporting the Serial Bootloader Interface
/// (SBL).
pub struct Device<P> {
    family: Family,
    port: P,
}

impl<P> Device<P>
where
    P: SerialPort,
{
    /// Create a new `Device` from an already opened port.
    ///
    /// This will synchronize the baudrate with the device.
    ///
    /// # Note
    ///
    /// This functions expects the device to be already in bootloader mode,
    /// to enter bootloader please reset the device into this mode or use
    /// [`invoke_bootloader`] function to enter the bootloader on the device
    /// (on supported boards).
    pub fn new(port: P, family: Family) -> io::Result<Self> {
        let mut device = Device { port, family };

        device.init_communications()?;

        Ok(device)
    }

    /// Returns the `Family` of the device.
    pub fn family(&self) -> Family {
        self.family
    }

    fn write_cmd<D>(&mut self, cmd: u8, data: &D) -> io::Result<()>
    where
        D: AsRef<[u8]>,
    {
        // [len | checksum | cmd]
        const HDR_LEN: usize = 3;

        let data = data.as_ref();

        let pkt_len = HDR_LEN + data.len();
        if pkt_len > usize::from(std::u8::MAX) {
            // Logic error, just panic.
            panic!("packet too big");
        }

        let mut pkt = Vec::with_capacity(pkt_len);

        pkt.push(pkt_len as u8);
        pkt.push(command_checksum(cmd, data));
        pkt.push(cmd);
        pkt.extend_from_slice(data);

        log::trace!("sending cmd {:#X}, pkt = {:?}", cmd, pkt);

        self.port.write_all(pkt.as_slice())?;
        self.port.flush()?;

        Ok(())
    }

    fn read_ack(&mut self) -> io::Result<bool> {
        log::trace!("waiting for ACK");

        let start_time = Instant::now();
        let timeout = Duration::from_secs(1);
        let mut ack = vec![0xFF, 0xFF];
        loop {
            let mut byte = [0u8; 1];
            match self.port.read(&mut byte) {
                Ok(n) if n == 0 => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "unexpected EOF",
                    ));
                }
                Ok(_) => {
                    ack.push(byte[0]);
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                    log::trace!("read timed out");
                }
                Err(e) => return Err(e),
            }

            if ack[ack.len() - 2] == 0x00
                && (ack[ack.len() - 1] == constants::ACK
                    || ack[ack.len() - 1] == constants::NACK)
            {
                log::trace!("ACK bytes found {:?}", &ack[2..]);
                break;
            } else if Instant::now().duration_since(start_time) >= timeout {
                log::trace!("ACK bytes not found, timed out");
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "ACK bytes not found, timed out",
                ));
            }
        }

        log::trace!("found ACK bytes after {} bytes", ack.len() - 2);

        match (ack[ack.len() - 2], ack[ack.len() - 1]) {
            (0x00, constants::ACK) => Ok(true),
            (0x00, constants::NACK) => Ok(false),
            _ => unreachable!(),
        }
    }

    fn write_ack(&mut self, ack: bool) -> io::Result<()> {
        let data: [u8; 2] =
            [0x00, if ack { constants::ACK } else { constants::NACK }];

        self.port.write_all(&data)?;
        self.port.flush()?;

        Ok(())
    }

    fn read_response(&mut self, response: &mut [u8]) -> io::Result<()> {
        const HDR_LEN: usize = 2;

        log::trace!("waiting for response header");
        let mut hdr = [0u8; HDR_LEN];
        self.port.read_exact(&mut hdr)?;
        log::trace!(
            "response header received, len = {}, cksum = {:#X}",
            hdr[0],
            hdr[1]
        );

        let payload_len = hdr[0] as usize - HDR_LEN;
        match response.len().cmp(&payload_len) {
            Ordering::Greater => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "received response is too small, expected {}, found {}",
                        response.len(),
                        payload_len,
                    ),
                ))
            }
            Ordering::Less => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "received response is too big, expected {}, found {}",
                        response.len(),
                        payload_len,
                    ),
                ))
            }
            _ => (),
        }

        log::trace!(
            "waiting for rest of response, expecting {} bytes",
            response.len()
        );
        self.port.read_exact(response)?;

        Ok(())
    }

    /// # Errors
    ///
    /// This function will return an error with the
    /// [`std::io::ErrorKind::NotConnected`] if the auto baud procedure didn't
    /// finish.
    fn perform_auto_baud(&mut self) -> io::Result<()> {
        // To synchronize with the host (us) send two bytes containing 0x55. If
        // synchronization succeeds, the bootloader will return an acknowledge.
        let data = [0x55u8, 0x55u8];
        self.port.write_all(&data)?;
        if !self.read_ack()? {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "couldn't synchronize bootloader baudrate",
            ));
        }

        log::debug!("Auto baud finished correctly");

        Ok(())
    }

    fn init_communications(&mut self) -> io::Result<()> {
        log::debug!("Sending dummy test command to check communication");
        self.write_cmd(0, &[])?;
        if self.read_ack().is_err() {
            log::debug!("No response received, performing auto baud procedure");
            // No successful response received, try auto baud.
            self.perform_auto_baud()?;
        }

        Ok(())
    }

    /// Ping the bootloader.
    pub fn ping(&mut self) -> io::Result<bool> {
        self.write_cmd(constants::CMD_PING, &[])?;
        self.read_ack()
    }

    /// Prepares flash programming.
    ///
    /// # Notes
    ///
    /// This command must be followed by a [`Device::get_status`] command
    /// to verify it worked.
    pub fn download(
        &mut self,
        program_address: u32,
        program_size: u32,
    ) -> io::Result<()> {
        const CMD_DOWNLOAD_LEN: usize = 8;

        let mut data = [0u8; CMD_DOWNLOAD_LEN];
        (&mut data[..4]).copy_from_slice(&program_address.to_be_bytes());
        (&mut data[4..]).copy_from_slice(&program_size.to_be_bytes());

        self.write_cmd(constants::CMD_DOWNLOAD, &data)?;
        let ack = self.read_ack()?;
        if !ack {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "COMMAND_DOWNLOAD not acknowledged",
            ));
        }

        Ok(())
    }

    /// Get the status of the last issued command.
    pub fn get_status(&mut self) -> io::Result<u8> {
        self.write_cmd(constants::CMD_GET_STATUS, &[])?;
        let ack = self.read_ack()?;
        if !ack {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "COMMAND_GET_STATUS not acknowledged",
            ));
        }

        let mut response = [0u8; 1];
        self.read_response(&mut response)?;
        self.write_ack(true)?;

        Ok(response[0])
    }

    /// Send data to be written into the flash memory.
    ///
    /// The return value represents if the command was
    /// acknowledged or not, if not acknowledged the write address
    /// is not incrmeneted by the device which allows for retransmissions
    /// of the previous data.
    ///
    /// # Note
    ///
    /// This only does work after a [`Device::download`] command
    /// has been issued.
    ///
    /// After issuing this command, [`Device::get_status`] command
    /// _should_ be used to check for errors.
    ///
    /// # Panics
    ///
    /// This function will panic if the `data` length in bytes is
    /// higher than [`constans::MAX_BYTES_PER_TRANSFER`].
    pub fn send_data<D>(&mut self, data: &D) -> io::Result<bool>
    where
        D: AsRef<[u8]>,
    {
        assert!(data.as_ref().len() <= constants::MAX_BYTES_PER_TRANSFER);

        self.write_cmd(constants::CMD_SEND_DATA, data)?;
        self.read_ack()
    }

    /// Read chip ID.
    pub fn get_chip_id(&mut self) -> io::Result<u32> {
        const CHIP_ID_RESPONSE_LEN: usize = 4;

        self.write_cmd(constants::CMD_GET_CHIP_ID, &[])?;
        let ack = self.read_ack()?;
        if !ack {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "bootloader didn't acknowledge the command",
            ));
        }

        let mut response = [0u8; CHIP_ID_RESPONSE_LEN];
        self.read_response(&mut response)?;
        self.write_ack(true)?;

        Ok(u32::from_be_bytes(response))
    }

    /// Erase. Only supported on [`Family::CC2538`].
    ///
    /// - See [`Family::supports_erase`].
    pub fn erase(&mut self, address: u32, byte_count: u32) -> io::Result<()> {
        const CMD_ERASE_LEN: usize = 8;

        if !self.family.supports_erase() {
            panic!("`COMMAND_ERASE` is not supported");
        }

        let mut data = [0u8; CMD_ERASE_LEN];
        (&mut data[..4]).copy_from_slice(&address.to_be_bytes());
        (&mut data[4..]).copy_from_slice(&byte_count.to_be_bytes());

        self.write_cmd(constants::CC2538_CMD_ERASE, &data)?;
        let ack = self.read_ack()?;
        if !ack {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to erase",
            ));
        }

        Ok(())
    }

    /// Sector erase. Only supported on [`Family::CC26X0`] and [`Family::CC26X2`].
    ///
    /// The size of each sector is specified at [`Family::sector_size`].
    ///
    /// - See [`Family::supports_sector_erase`].
    ///
    /// # Parameters
    ///
    /// - `address`: The start address of the sector.
    ///
    /// # Panics
    ///
    /// - This function panics if the family doesn't support this command.
    /// - This function panics if the `address` isn't the start of a sector.
    ///
    /// See [`util::erase_flash_range`] for an easier to use wrapper of this
    /// function.
    pub fn sector_erase(&mut self, address: u32) -> io::Result<()> {
        const CMD_SECTOR_ERASE_LEN: usize = 4;

        if !self.family.supports_sector_erase() {
            panic!("`COMMAND_SECTOR_ERASE` is not supported");
        }

        assert!(
            address % self.family.sector_size() == 0,
            "invalid sector address"
        );

        let mut data = [0u8; CMD_SECTOR_ERASE_LEN];
        data.copy_from_slice(&address.to_be_bytes());

        self.write_cmd(constants::CC26X0_CMD_SECTOR_ERASE, &data)?;
        let ack = self.read_ack()?;
        if !ack {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to erase sector",
            ));
        }

        Ok(())
    }

    /// Switch to XOSC. Only supported on [`Family::CC2538`].
    ///
    /// - See [`Family::supports_set_xosc`].
    ///
    /// # Panics
    ///
    /// This function panics if the family doesn't support this command.
    pub fn set_xosc(&mut self) -> io::Result<()> {
        if !self.family.supports_set_xosc() {
            panic!("XOSC switch is not supported");
        }

        self.write_cmd(constants::CC2538_CMD_SET_XOSC, &[])?;
        let ack = self.read_ack()?;
        if !ack {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to switch to xosc",
            ));
        }

        Ok(())
    }

    /// Read memory using 32-bit access type.
    ///
    /// # Parameters
    ///
    /// - `address`: the memory address to read. Must be aligned to 32-bits.
    /// - `data`: where the data will be stored. Can't be higher than `63 * 4`
    /// bytes. The number of bytes MUST be exactly divisible by 4.
    ///
    /// # Panics
    ///
    /// - This function will panic if the length of the `data` slice
    /// is higher than `63 * 4` bytes, this is the maximum number of accesses
    /// that can be done using this mode.
    /// - This function will panic if the `address` is not aligned to 32-bits.
    ///
    /// See [`util::memory_read_32`] for an easy to use version of this function.
    pub fn memory_read_32(
        &mut self,
        address: u32,
        data: &mut [u8],
    ) -> io::Result<()> {
        const MEMORY_READ_LEN: usize = 6;
        if let Family::CC2538 = self.family {
            panic!("32-bit memory accesses are only allowed on CC26xx");
        }

        assert!(
            data.len() <= (63 * 4),
            "only a maximum of 63 accesses can be done on word mode"
        );
        assert!(
            data.len() % 4 == 0,
            "number of bytes is not divisible from 4"
        );
        assert!(
            (address & 0x03) == 0,
            "memory address must be 32-bits aligned"
        );

        log::trace!(
            "memory_read_32 `{}` elements at start address `{:#X}`",
            data.len() / 4,
            address
        );

        let mut cmd = [0u8; MEMORY_READ_LEN];
        (&mut cmd[..4]).copy_from_slice(&address.to_be_bytes()); /* address */
        cmd[4] = 1; /* access type */
        cmd[5] = (data.len() / 4) as u8; /* number of accesses */
        self.write_cmd(constants::CMD_MEMORY_READ, &cmd)?;
        let ack = self.read_ack()?;
        if !ack {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to read memory",
            ));
        }

        self.read_response(data)?;
        self.write_ack(true)?;

        Ok(())
    }
}

impl<P> fmt::Debug for Device<P>
where
    P: SerialPort,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Device")
            .field("family", &self.family)
            .field("port", &())
            .finish()
    }
}

fn command_checksum(cmd: u8, data: &[u8]) -> u8 {
    let mut checksum: u8 = cmd;
    for byte in data {
        checksum = checksum.overflowing_add(*byte).0;
    }

    checksum
}

/// Default serial port settings.
///
/// It's recommended to change only the baudrate since all other
/// options are the same for all Texas Instruments devices.
pub fn port_settings() -> serial::PortSettings {
    serial::PortSettings {
        baud_rate: serial::BaudRate::Baud115200,
        char_size: serial::CharSize::Bits8,
        parity: serial::Parity::ParityNone,
        stop_bits: serial::StopBits::Stop1,
        flow_control: serial::FlowControl::FlowNone,
    }
}

/// Use the DTR and RTS lines to control bootloader and the !RESET pin.
/// This can automatically invoke the bootloader without the user
/// having to toggle any pins.
///
/// # Parameters:
///
/// - `inverted`: if it's `false` (default) DTR is connected to the bootloader pin,
/// RTS connnected to !RESET. If it's `true` it's the other way around
/// - `bootloader_active_high`: whether the bootloader pin used is active low or
/// active high.
#[allow(clippy::needless_bool)]
pub fn invoke_bootloader<P>(
    port: &mut P,
    inverted: bool,
    bootloader_active_high: bool,
) -> serial::Result<()>
where
    P: SerialPort,
{
    fn set_bootloader_pin<P: SerialPort>(
        port: &mut P,
        inverted: bool,
        level: bool,
    ) -> serial::Result<()> {
        if inverted {
            port.set_rts(level)
        } else {
            port.set_dtr(level)
        }
    }

    fn set_reset_pin<P: SerialPort>(
        port: &mut P,
        inverted: bool,
        level: bool,
    ) -> serial::Result<()> {
        if inverted {
            port.set_dtr(level)
        } else {
            port.set_rts(level)
        }
    }

    set_bootloader_pin(
        port,
        inverted,
        if !bootloader_active_high { true } else { false },
    )?;
    set_reset_pin(port, inverted, false)?;
    set_reset_pin(port, inverted, true)?;
    set_reset_pin(port, inverted, false)?;
    // Make sure the pin is still asserted when the chip comes out of reset.
    #[cfg(not(test))]
    std::thread::sleep(Duration::from_millis(2));
    set_bootloader_pin(
        port,
        inverted,
        if !bootloader_active_high { false } else { true },
    )?;

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_command_checksum() {
        // nonsensical data, just to make sure it works.
        const DATA: &[u8] = &[0xde, 0xad, 0xbe, 0xef];
        assert_eq!(command_checksum(0xCA, DATA), 0x02);
    }

    #[test]
    #[allow(bare_trait_objects)]
    fn test_invoke_bootloader() {
        struct DummySerialPort {
            rts_state: bool,
            dtr_state: bool,
        }

        impl SerialPort for DummySerialPort {
            fn timeout(&self) -> Duration {
                unreachable!()
            }
            fn set_timeout(
                &mut self,
                _timeout: Duration,
            ) -> serial::Result<()> {
                unreachable!()
            }
            fn configure(
                &mut self,
                _settings: &serial::PortSettings,
            ) -> serial::Result<()> {
                unreachable!()
            }
            fn reconfigure(
                &mut self,
                _setup: &Fn(
                    &mut serial::SerialPortSettings,
                ) -> serial::Result<()>,
            ) -> serial::Result<()> {
                unreachable!()
            }
            fn set_rts(&mut self, level: bool) -> serial::Result<()> {
                self.rts_state = level;
                Ok(())
            }
            fn set_dtr(&mut self, level: bool) -> serial::Result<()> {
                self.dtr_state = level;
                Ok(())
            }
            fn read_cts(&mut self) -> serial::Result<bool> {
                unreachable!()
            }
            fn read_dsr(&mut self) -> serial::Result<bool> {
                unreachable!()
            }
            fn read_ri(&mut self) -> serial::Result<bool> {
                unreachable!()
            }
            fn read_cd(&mut self) -> serial::Result<bool> {
                unreachable!()
            }
        }

        impl io::Read for DummySerialPort {
            fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
                unreachable!()
            }
        }

        impl io::Write for DummySerialPort {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                unreachable!()
            }
            fn flush(&mut self) -> io::Result<()> {
                unreachable!()
            }
        }

        // Initial state
        let mut port = DummySerialPort {
            rts_state: false,
            dtr_state: false,
        };

        // Test that the invoke functionality leaves the pins on their normal level.
        invoke_bootloader(&mut port, false, false).unwrap();
        assert_eq!(port.rts_state, false);
        assert_eq!(port.dtr_state, false);

        // Reset values.
        port.rts_state = false;
        port.dtr_state = false;

        // Test that the invoke functionality leaves the pins on their normal level.
        invoke_bootloader(&mut port, true, false).unwrap();
        assert_eq!(port.rts_state, false);
        assert_eq!(port.dtr_state, false);

        // Reset values, now for active-high.
        port.rts_state = false;
        port.dtr_state = true;

        // Test that the invoke functionality leaves the pins on their normal level.
        invoke_bootloader(&mut port, false, true).unwrap();
        assert_eq!(port.rts_state, false);
        assert_eq!(port.dtr_state, true);

        // Reset values, now for active-high and inverted.
        port.rts_state = true;
        port.dtr_state = false;

        // Test that the invoke functionality leaves the pins on their normal level.
        invoke_bootloader(&mut port, true, true).unwrap();
        assert_eq!(port.rts_state, true);
        assert_eq!(port.dtr_state, false);
    }
}
