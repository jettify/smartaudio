#![no_std]
#![allow(clippy::needless_doctest_main)]
#![doc = include_str!("../README.md")]
pub mod commands;
pub(crate) mod constants;
pub mod parser;
pub mod responses;

//Command frames
pub use commands::GetSettingsCommand;
pub use commands::SetChannelCommand;
pub use commands::SetFrequencyCommand;
pub use commands::SetModeCommand;
pub use commands::SetPowerCommand;
pub use commands::SmartAudioCommand;

// Response frames
pub use responses::Response;
pub use responses::SetChannelResponse;
pub use responses::SetFrequencyResponse;
pub use responses::SetModeResponse;
pub use responses::SetPowerResponse;
pub use responses::Settings;
pub use responses::SmartAudioReponse;
// Parsing
pub use parser::RawSmartAudioFrame;
pub use parser::SmartAudioError;
pub use parser::SmartAudioParser;
