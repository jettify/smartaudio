#![no_std]
#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]
pub mod frames;
pub mod parser;

pub use parser::RawSmartAudioFrame;
pub use parser::SmartAudioError;
pub use parser::SmartAudioParser;
