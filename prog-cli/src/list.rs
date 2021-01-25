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

use ti_sbl::ports::PortInfo;

use anyhow::Result;

pub fn list() -> Result<()> {
    let ports = PortInfo::list_all();

    for port in ports {
        if let Some(usb_info) = port.usb_info {
            match (usb_info.manufacturer, usb_info.product) {
                (Some(manufacturer), Some(product)) => {
                    println!(
                        "- `{}` {:04X}:{:04X} {} {}",
                        port.port.to_string_lossy(),
                        usb_info.vid,
                        usb_info.pid,
                        manufacturer,
                        product
                    );
                }
                (Some(manufacturer), None) => {
                    println!(
                        "- `{}` {:04X}:{:04X} {}",
                        port.port.to_string_lossy(),
                        usb_info.vid,
                        usb_info.pid,
                        manufacturer,
                    );
                }
                (None, Some(product)) => {
                    println!(
                        "- `{}` {:04X}:{:04X} {}",
                        port.port.to_string_lossy(),
                        usb_info.vid,
                        usb_info.pid,
                        product,
                    );
                }
                _ => {
                    println!(
                        "- `{}` {:04X}:{:04X}",
                        port.port.to_string_lossy(),
                        usb_info.vid,
                        usb_info.pid,
                    );
                }
            }
        } else {
            println!("- `{}`", port.port.to_string_lossy());
        }
    }

    Ok(())
}
