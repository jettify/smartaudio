use crate::constants::get_settings_flags;
use crate::constants::mode_flags;
use crate::constants::response as resp;
use crate::{parser::SmartAudioError, RawSmartAudioFrame};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Version {
    V1_0,
    #[default]
    V2_0,
    V2_1,
    Unknown,
}

impl From<u8> for Version {
    fn from(v: u8) -> Self {
        match v {
            // Command byte for getting settings also indicates
            // versoin of protocol.
            resp::GET_SETTINGS_V1_0 => Self::V1_0,
            resp::GET_SETTINGS_V2_0 => Self::V2_0,
            resp::GET_SETTINGS_V2_1 => Self::V2_1,
            _ => Self::Unknown,
        }
    }
}

pub trait SmartAudioReponse {
    fn from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Self;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PowerSettings {
    pub current_power: u8,
    pub num_power_levels: u8,
    pub dbm_level_1: u8,
    pub dbm_level_2: u8,
    pub dbm_level_3: u8,
    pub dbm_level_4: u8,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Settings {
    pub version: Version,
    pub channel: u8,
    pub power_level: u8,
    pub frequency: u16,
    pub unlocked: bool,
    pub user_frequency_mode: bool,
    pub pitmode_enabled: bool,
    pub pitmode_in_range_active: bool,
    pub pitmode_out_range_active: bool,
    pub power_settings: Option<PowerSettings>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetPowerResponse {
    power: u8,
}

impl SmartAudioReponse for SetPowerResponse {
    fn from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Self {
        Self {
            power: raw_frame.payload()[0],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetChannelResponse {
    channel: u8,
}

impl SmartAudioReponse for SetChannelResponse {
    fn from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Self {
        Self {
            channel: raw_frame.payload()[0],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetFrequencyResponse {
    frequency: u16,
}

impl SmartAudioReponse for SetFrequencyResponse {
    fn from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Self {
        let buffer = raw_frame.payload();
        Self {
            frequency: u16::from_be_bytes([buffer[0], buffer[1]]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetModeResponse {
    pitmode_in_range_active: bool,
    pitmode_out_range_active: bool,
    pitmode_enabled: bool,
    unlocked: bool,
}

impl SmartAudioReponse for SetModeResponse {
    fn from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Self {
        let mode = raw_frame.payload()[0];
        Self {
            pitmode_in_range_active: mode & mode_flags::PITMODE_IN_RANGE != 0,
            pitmode_out_range_active: mode & mode_flags::PITMODE_OUT_RANGE != 0,
            pitmode_enabled: mode & mode_flags::PITMODE_ENABLED != 0,
            unlocked: mode & mode_flags::UNLOCKED != 0,
        }
    }
}

impl SmartAudioReponse for Settings {
    fn from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Self {
        let b = raw_frame.payload();

        let version = Version::from(raw_frame.commnand());
        let channel = b[0];
        let power_level = b[1];

        // unpack mode
        let mode = b[2];
        let pitmode_enabled = mode & get_settings_flags::PITMODE_ENABLED != 0;
        let pitmode_in_range_active = mode & get_settings_flags::PITMODE_IN_RANGE != 0;
        let pitmode_out_range_active = mode & get_settings_flags::PITMODE_OUT_RANGE != 0;
        let unlocked = mode & get_settings_flags::UNLOCKED != 0;
        let user_frequency_mode = mode & get_settings_flags::USER_FREQUENCY != 0;

        let frequency = u16::from_be_bytes([b[3], b[4]]);

        let power_settings = if version == Version::V2_1 {
            Some(PowerSettings {
                current_power: b[5],
                num_power_levels: b[6],
                dbm_level_1: b[7],
                dbm_level_2: b[8],
                dbm_level_3: b[9],
                dbm_level_4: b[10],
            })
        } else {
            None
        };

        Self {
            version,
            channel,
            power_level,
            frequency,
            pitmode_enabled,
            pitmode_in_range_active,
            pitmode_out_range_active,
            unlocked,
            user_frequency_mode,
            power_settings,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Response {
    GetSettings(Settings),
    SetPower(SetPowerResponse),
    SetChannel(SetChannelResponse),
    SetFrequency(SetFrequencyResponse),
    SetMode(SetModeResponse),
    Unknown(u8),
}

impl Response {
    pub fn parse(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, SmartAudioError> {
        let cmd = raw_frame.commnand();
        match cmd {
            resp::GET_SETTINGS_V1_0 | resp::GET_SETTINGS_V2_0 | resp::GET_SETTINGS_V2_1 => {
                Ok(Self::GetSettings(Settings::from_raw_frame(raw_frame)))
            }
            resp::SET_POWER => Ok(Self::SetPower(SetPowerResponse::from_raw_frame(raw_frame))),
            resp::SET_CHANNEL => Ok(Self::SetChannel(SetChannelResponse::from_raw_frame(
                raw_frame,
            ))),
            resp::SET_FREQUENCY => Ok(Self::SetFrequency(SetFrequencyResponse::from_raw_frame(
                raw_frame,
            ))),
            resp::SET_MODE => Ok(Self::SetMode(SetModeResponse::from_raw_frame(raw_frame))),
            _ => Err(SmartAudioError::InvalidHeader),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::RawSmartAudioFrame;

    #[test]
    fn test_get_settings_v1_0_parsing() {
        let raw: [u8; 10] = [0xAA, 0x55, 0x01, 0x06, 0x00, 0x00, 0x01, 0x16, 0xE9, 0x4D];
        let frame = RawSmartAudioFrame::new(&raw).unwrap();
        let packet = Response::parse(&frame).unwrap();
        let expected = Settings {
            version: Version::V1_0,
            channel: 0,
            power_level: 0,
            frequency: 5865,
            unlocked: false,
            user_frequency_mode: true,
            pitmode_enabled: false,
            pitmode_in_range_active: false,
            pitmode_out_range_active: false,
            power_settings: None,
        };
        assert!(matches!(packet, Response::GetSettings(actual) if actual == expected));
    }

    #[test]
    fn test_get_settings_v2_0_parsing() {
        let raw: [u8; 10] = [0xAA, 0x55, 0x09, 0x06, 0x01, 0x00, 0x1A, 0x16, 0xE9, 0x0A];
        let frame = RawSmartAudioFrame::new(&raw).unwrap();
        let packet = Response::parse(&frame).unwrap();
        let expected = Settings {
            version: Version::V2_0,
            channel: 1,
            power_level: 0,
            frequency: 5865,
            unlocked: true,
            user_frequency_mode: false,
            pitmode_enabled: true,
            pitmode_in_range_active: false,
            pitmode_out_range_active: true,
            power_settings: None,
        };
        assert!(matches!(packet, Response::GetSettings(actual) if actual == expected));
    }

    #[test]
    fn test_get_settings_v2_1_parsing() {
        let raw: [u8; 16] = [
            0xAA, 0x55, 0x11, 0x0C, 0x00, 0x00, 0x00, 0x16, 0xE9, 0x0E, 0x03, 0x00, 0x0E, 0x14,
            0x1A, 0x01,
        ];
        let frame = RawSmartAudioFrame::new(&raw).unwrap();
        let packet = Response::parse(&frame).unwrap();
        let expected = Settings {
            version: Version::V2_1,
            channel: 0,
            power_level: 0,
            frequency: 5865,
            unlocked: false,
            user_frequency_mode: false,
            pitmode_enabled: false,
            pitmode_in_range_active: false,
            pitmode_out_range_active: false,
            power_settings: Some(PowerSettings {
                current_power: 14,
                num_power_levels: 3,
                dbm_level_1: 0,
                dbm_level_2: 14,
                dbm_level_3: 20,
                dbm_level_4: 26,
            }),
        };
        assert!(matches!(packet, Response::GetSettings(actual) if actual == expected));
    }

    #[test]
    fn test_set_power_response_parsing() {
        let raw_v20: [u8; 7] = [0xAA, 0x55, 0x02, 0x03, 0x00, 0x01, 0x0F];
        let frame_v20 = RawSmartAudioFrame::new(&raw_v20).unwrap();
        let packet_v20 = Response::parse(&frame_v20).unwrap();
        let expected = SetPowerResponse { power: 0 };
        assert!(matches!(packet_v20, Response::SetPower(actual) if actual == expected));

        let raw_v21: [u8; 7] = [0xAA, 0x55, 0x02, 0x03, 0x0E, 0x01, 0x6D];
        let frame_v21 = RawSmartAudioFrame::new(&raw_v21).unwrap();
        let packet_v21 = Response::parse(&frame_v21).unwrap();
        let expected = SetPowerResponse { power: 14 };
        assert!(matches!(packet_v21, Response::SetPower(actual) if actual == expected));
    }

    #[test]
    fn test_set_channel_response_parsing() {
        let raw: [u8; 7] = [0xAA, 0x55, 0x03, 0x03, 0x00, 0x01, 0x4A];
        let frame = RawSmartAudioFrame::new(&raw).unwrap();
        let packet = Response::parse(&frame).unwrap();
        let expected = SetChannelResponse { channel: 0 };
        assert!(matches!(packet, Response::SetChannel(actual) if actual == expected));
    }

    #[test]
    fn test_set_frequency_response_parsing() {
        let raw: [u8; 8] = [0xAA, 0x55, 0x04, 0x04, 0x16, 0xE9, 0x01, 0xF8];
        let frame = RawSmartAudioFrame::new(&raw).unwrap();
        let packet = Response::parse(&frame).unwrap();
        let expected = SetFrequencyResponse { frequency: 5865 };
        assert!(matches!(packet, Response::SetFrequency(actual) if actual == expected));
    }

    #[test]
    fn test_set_mode_response_parsing() {
        let raw: [u8; 7] = [0xAA, 0x55, 0x05, 0x03, 0x0A, 0x01, 0x4F];
        let frame = RawSmartAudioFrame::new(&raw).unwrap();
        let packet = Response::parse(&frame).unwrap();

        let expected = SetModeResponse {
            pitmode_in_range_active: false,
            pitmode_out_range_active: true,
            pitmode_enabled: false,
            unlocked: true,
        };
        assert!(matches!(packet, Response::SetMode(actual) if actual == expected));
    }
}
