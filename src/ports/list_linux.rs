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

use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use super::{PortInfo, PortUsbInfo};

fn glob(pat: &str) -> glob::Paths {
    glob::glob(pat).unwrap()
}

fn read_line<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = BufReader::new(File::open(path)?);

    let mut line = String::new();
    file.read_line(&mut line)?;

    Ok(line.trim().to_owned())
}

fn pathdir(mut path: PathBuf) -> PathBuf {
    path.pop();
    path
}

fn port_info<P>(port: P) -> io::Result<Option<PortInfo>>
where
    P: AsRef<Path>,
{
    let port = port.as_ref();

    let device_path = PathBuf::from("/sys/class/tty")
        .join(port.file_name().unwrap())
        .join("device");

    let subsystem = if device_path.exists() {
        fs::canonicalize(
            fs::canonicalize(device_path.clone())?.join("subsystem"),
        )?
        .file_name()
        .map(|s| s.to_owned())
    } else {
        None
    };

    let usb_int = if let Some(ref subsystem) = subsystem {
        // We don't want internal serial ports.
        if subsystem == "platform" {
            return Ok(None);
        } else if subsystem == "usb-serial" {
            Some(pathdir(fs::canonicalize(device_path)?))
        } else if subsystem == "usb" {
            Some(device_path)
        } else {
            None
        }
    } else {
        None
    };

    let usb_info = if let Some(usb_int) = usb_int {
        let usb_dev = pathdir(usb_int.clone());

        Some(PortUsbInfo {
            num_if: read_line(usb_dev.join("bNumInterfaces"))
                .unwrap_or_else(|_| "1".to_owned())
                .parse()
                .unwrap_or(1),
            vid: u16::from_str_radix(&read_line(usb_dev.join("idVendor"))?, 16)
                .unwrap(),
            pid: u16::from_str_radix(
                &read_line(usb_dev.join("idProduct"))?,
                16,
            )
            .unwrap(),
            serial: read_line(usb_dev.join("serial")).ok(),
            manufacturer: read_line(usb_dev.join("manufacturer")).ok(),
            product: read_line(usb_dev.join("product")).ok(),
            interface: read_line(usb_int.join("interface")).ok(),
        })
    } else {
        None
    };

    Ok(Some(PortInfo {
        port: OsString::from(port),
        name: port.file_name().unwrap().to_owned(),
        usb_info,
    }))
}

pub fn list_all() -> Vec<PortInfo> {
    let mut ports = Vec::new();

    ports.extend(glob("/dev/ttyS*")); // Built-in serial ports
    ports.extend(glob("/dev/ttyUSB*")); // usb-serial with own driver
    ports.extend(glob("/dev/ttyXRUSB*")); // xr-usb-serial port exar (DELL Edge 3001)
    ports.extend(glob("/dev/ttyACM*")); // usb-serial with CDC-ACM profile
    ports.extend(glob("/dev/ttyAMA*")); // ARM internal port (raspi)
    ports.extend(glob("/dev/rfcomm*")); // BT serial devices
    ports.extend(glob("/dev/ttyAP*")); // Advantech multi-port serial controllers

    let mut available = Vec::new();
    for port in ports {
        if let Ok(ref port) = port {
            if let Ok(Some(info)) = port_info(port) {
                available.push(info);
            }
        }
    }

    available
}
