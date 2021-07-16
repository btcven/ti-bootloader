<p align="center">
  <a href="https://locha.io/">
  <img height="200px" src="doc/logo.png">
  </a>
</p>

<p align="center">
  <a href="https://locha.io/">Project Website</a>
</p>

<h1 align="center">TI bootloader</h1>

This is a command line tool and Rust library to flash/read Texas Instruments
microcontrollers that support the serial bootloader.

# Installation

Install Rust stable using your package manager or preferably, from [rustup.rs].

[rustup.rs]: https://rustup.rs/

And clone the repository with:

```
git clone https://github.com/btcven/ti-bootloader
```

Build and install the binary with:

```
cargo install --path ti-bootloader/prog-cli
```

Your binary will be installed in the `.cargo/bin` directory, depending on your os,
make sure it is on your `PATH` environment variable.

To display the usage:

```
ti-sbl-prog --help
```

# Flashing a binary

This command will flash a binary (`hello-world.bin`) onto your device (make
sure it's on bootloader mode), using a baud rate of 1.5 mbauds, for the
`cc26x2` family of MCUs, and erasing sectors before writing.

```
ti-sbl-prog -p /dev/ttyUSB0 flash hello-world.bin --write-erase --family cc26x2 --baudrate 1500000
```

# [Documentation](https://btcven.github.io/ti-bootloader/ti_sbl/index.html)

# License

```
Copyright 2021 btcven and Locha Mesh Developers <contact@locha.io>

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
