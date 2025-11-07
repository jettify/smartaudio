# SmartAudio
[![CI](https://github.com/jettify/smartaudio/actions/workflows/CI.yml/badge.svg)](https://github.com/jettify/smartaudio/actions/workflows/CI.yml)
[![codecov](https://codecov.io/gh/jettify/smartaudio/graph/badge.svg?token=RCM2W4C0LB)](https://codecov.io/gh/jettify/smartaudio)
[![crates.io](https://img.shields.io/crates/v/smartaudio)](https://crates.io/crates/smartaudio)
[![docs.rs](https://img.shields.io/docsrs/smartaudio)](https://docs.rs/smartaudio/latest/smartaudio/)

This is a `no_std` platform-agnostic implementation of the TBS `SmartAudio` protocol in Rust.

## Features

* `no_std`, does not use the standard library or an allocator.
* Platform Agnostic, can be used on any MCU or platform.
* Provides a low-level interface to slice byte stream into valid frames.
* Supports `SmartAudio` protocols `1.0`, `2.0` and `2.1`.

## Usage Example

Here is a basic example of how to parse buffer using iterator:

```rust

use smartaudio::SmartAudioParser;

fn main() {
    let raw: [u8; 72] = [
        0xAA, 0x55, 0x01, 0x06, 0x00, 0x00, 0x01, 0x16, 0xE9, 0x4D, // frame0
        0xAA, 0x55, 0x09, 0x06, 0x01, 0x00, 0x1A, 0x16, 0xE9, 0x0A, // frome1
        0xAA, 0x55, 0x11, 0x0C, 0x00, 0x00, 0x00, 0x16, 0xE9, 0x0E, 0x03, 0x00, 0x0E, 0x14, 0x1A,
        0x01, // frame2
        0xAA, 0x55, 0x02, 0x03, 0x00, 0x01, 0x0F, // frame3
        0xAA, 0x55, 0x02, 0x03, 0x0E, 0x01, 0x6D, // frame4
        0xAA, 0x55, 0x03, 0x03, 0x00, 0x01, 0x4A, // frame5
        0xAA, 0x55, 0x04, 0x04, 0x16, 0xE9, 0x01, 0xF8, // frame6
        0xAA, 0x55, 0x05, 0x03, 0x0A, 0x01, 0x4F, // frame7
    ];

    println!("Parssing frame from buffer using iterator:");

    let mut parser = SmartAudioParser::new();
    for frame in parser.iter_responses(&raw[..]) {
        println!("{:#?}", frame.unwrap())
    }
}
```

Here is a basic example feeding data stream one byte at time:

```rust
use smartaudio::Response;
use smartaudio::SmartAudioParser;

fn main() {
    let mut parser = SmartAudioParser::new();

    let raw_settings_v20: [u8; 10] = [
        0xAA, 0x55, // Headers
        0x09, // Version/Command
        0x06, // Length
        0x01, // Channel
        0x00, // Power Level
        0x1A, // Operation/Mode
        0x16, 0xE9, // Current Frequency 5865
        0x0A, // CRC8
    ];
    println!("Parssing raw packet:");
    for b in &raw_settings_v20[0..raw_settings_v20.len()] {
        let reult = parser.push_byte(*b);
        match reult {
            Ok(Some(Response::GetSettings(settings))) => {
                println!("{:#?}", settings);
                println!("\n");
                println!("Version:     {:?}", settings.version);
                println!("Channel:     {:?}", settings.channel);
                println!("Power Level: {:?}", settings.power_level);
                println!("Unlocked:    {:?}", settings.unlocked);
            }
            Err(e) => eprintln!("Error parsing packet: {:?}", e),
            _ => (),
        }
    }
}

```

## Installation

Add `smartaudio` to your `Cargo.toml`:

```toml
[dependencies]
smartaudio = "*" # replace * by the latest version of the crate.
```

Or use the command line:

```bash
cargo add smartaudio
```

## License

This project is licensed under the `Apache 2.0`. See the [LICENSE](https://github.com/jettify/smartaudio/blob/master/LICENSE) file for details.

## Protocol Specification

[tbs_smartaudio_rev09.pdf](https://www.team-blacksheep.com/tbs_smartaudio_rev09.pdf)
