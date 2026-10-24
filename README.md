# `SmartAudio`
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

## Note

Originally developed for the `uflight` flight controller project, but useful as a standalone crate.
Library is under active development and testing, API might change at any time.

## Usage Example

Here is a basic example of how to parse buffer using iterator:

```rust

use smartaudio::SmartAudioParser;

fn main() {
    let raw: [u8; 72] = [
        0xAA, 0x55, 0x01, 0x06, 0x00, 0x00, 0x01, 0x16, 0xE9, 0x4D, // frame0
        0xAA, 0x55, 0x09, 0x06, 0x01, 0x00, 0x1A, 0x16, 0xE9, 0x0A, // frame1
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

Complete frames are parsed directly from the input slice. Frames split across
chunks use an internal 32-byte partial-frame buffer. The parser retains no input references
between calls, so transport buffers can be reused after their frames are consumed.
Typed responses remain owned, including the V2.1 power table.

For raw frames, use `next_frame` with a mutable slice cursor. It advances the cursor
on both frames and errors. `Ok(None)` means the chunk is exhausted; any partial
frame remains buffered. Consume each returned frame before calling again.

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
    // A transport may return chunks ending anywhere within a frame.
    for chunk in raw_settings_v20.chunks(4) {
        let mut input = chunk;
        loop {
            match parser.next_frame(&mut input) {
                Ok(Some(frame)) => println!("{:#?}", frame),
                Ok(None) => break,
                Err(error) => eprintln!("Error parsing packet: {:?}", error),
            }
        }
    }
}
```

Call `reset()` after a timeout or transport interruption to discard a partial
frame. The slice APIs replace `push_byte` and `push_byte_raw`; for a single byte,
pass `core::slice::from_ref(&byte)` as the chunk. When using `iter_responses`,
exhaust the iterator to retain a trailing partial frame; dropping it early leaves
its remaining bytes unconsumed.

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
