# SmartAudio
[![CI](https://github.com/jettify/smartaudio/actions/workflows/CI.yml/badge.svg)](https://github.com/jettify/smartaudio/actions/workflows/CI.yml)

This is a `no_std` platform-agnostic implementation of the TBS `SmartAudio` protocol in Rust.

## Features

*   `no_std`: Does not use the standard library or an allocator.
*   Platform Agnostic: Can be used on any MCU or platform.
*   Ergonomic: Designed to be easy to use in idiomatic Rust.
*   Layered: Provides a low-level layer for raw byte handling and a high-level layer for structured data.

## Usage Example

Here is a basic example of how to parse a raw `SmartAudio` frame:

```rust
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
    println!("Parssing packet:");
    for b in &raw_settings_v20[0..raw_settings_v20.len()] {
        let reult = parser.push_byte_raw(*b);
        match reult {
            Ok(Some(f)) => println!("{:?}", f),
            Err(e) => eprintln!("Error parsing packet: {:?}", e),
            _ => (),
        }
    }
}
```

## License

This project is licensed under the `GPL v3`. See the [LICENSE](https://github.com/jettify/smartaudio/blob/master/LICENSE) file for details.

## Protocol Specification

[tbs_smartaudio_rev09.pdf](https://www.team-blacksheep.com/tbs_smartaudio_rev09.pdf)
