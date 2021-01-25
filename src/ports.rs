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

use std::ffi::OsString;

#[cfg(target_os = "linux")]
mod list_linux;
#[cfg(target_os = "macos")]
mod list_macos;
#[cfg(target_os = "windows")]
mod list_windows;

/// Information about an available serial port.
#[derive(Debug)]
pub struct PortInfo {
    pub port: OsString,
    pub name: OsString,
    pub usb_info: Option<PortUsbInfo>,
}

impl PortInfo {
    /// List all serial ports on the system.
    #[cfg(target_os = "linux")]
    pub fn list_all() -> Vec<PortInfo> {
        self::list_linux::list_all()
    }

    #[cfg(target_os = "macos")]
    pub fn list_all() -> Vec<PortInfo> {
        self::list_macos::list_all()
    }

    #[cfg(target_os = "windows")]
    pub fn list_all() -> Vec<PortInfo> {
        self::list_windows::list_all()
    }
}

/// Information about USB serial ports.
#[derive(Debug)]
pub struct PortUsbInfo {
    /// Number of interfaces in this device.
    pub num_if: usize,
    /// USB Vendor ID.
    pub vid: u16,
    /// USB Product ID.
    pub pid: u16,
    /// Serial number string.
    pub serial: Option<String>,
    /// Device manufacturer.
    pub manufacturer: Option<String>,
    /// Device product description.
    pub product: Option<String>,
    /// Device product interface.
    pub interface: Option<String>,
}
