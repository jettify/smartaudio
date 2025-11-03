#![no_std]
#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]
pub mod commands;
pub(crate) mod constants;
pub mod frames;
pub mod parser;

//Command frames
pub use commands::GetSettingsCommand;
pub use commands::SetChannelCommand;
pub use commands::SetFrequencyCommand;
pub use commands::SetModeCommand;
pub use commands::SetPowerCommand;
pub use commands::SmartAudioCommand;

// Response frames
pub use frames::Response;
pub use frames::SetChannelResponse;
pub use frames::SetFrequencyResponse;
pub use frames::SetModeResponse;
pub use frames::SetPowerResponse;
pub use frames::Settings;
pub use frames::SmartAudioReponse;
// Parsing
pub use parser::RawSmartAudioFrame;
pub use parser::SmartAudioError;
pub use parser::SmartAudioParser;
