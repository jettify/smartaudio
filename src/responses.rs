use crate::constants::get_settings_flags;
use crate::constants::mode_flags;
use crate::constants::response as resp;
use crate::SmartAudioParser;
use crate::{parser::SmartAudioError, RawSmartAudioFrame};

/// The SmartAudio protocol version.
/// This is determined from the command byte of a `GetSettings` response.
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
            // Version of protocol.
            resp::GET_SETTINGS_V1_0 => Self::V1_0,
            resp::GET_SETTINGS_V2_0 => Self::V2_0,
            resp::GET_SETTINGS_V2_1 => Self::V2_1,
            _ => Self::Unknown,
        }
    }
}

pub trait SmartAudioResponse: Sized {
    fn try_from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, SmartAudioError>;
}

/// Detailed power settings, included in `GetSettings` response for SmartAudio V2.1+.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PowerSettings {
    /// The currently selected power in dBm.
    pub current_power: u8,
    /// The number of available power levels.
    pub num_power_levels: u8,
    /// Power level 1 in dBm.
    pub dbm_level_1: u8,
    /// Power level 2 in dBm.
    pub dbm_level_2: u8,
    /// Power level 3 in dBm.
    pub dbm_level_3: u8,
    /// Power level 4 in dBm.
    pub dbm_level_4: u8,
}

/// The VTX settings, returned in response to a `GetSettingsCommand`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Settings {
    /// SmartAudio protocol version.
    pub version: Version,
    /// Current channel (0-39).
    pub channel: u8,
    /// Current power level index.
    pub power_level: u8,
    /// Current frequency in MHz.
    pub frequency: u16,
    /// VTX is unlocked.
    pub unlocked: bool,
    /// VTX is in user-defined frequency mode.
    pub user_frequency_mode: bool,
    /// Pit mode is currently active.
    pub pitmode_enabled: bool,
    /// In-range pit mode is configured.
    pub pitmode_in_range_active: bool,
    /// Out-of-range pit mode is configured.
    pub pitmode_out_range_active: bool,
    /// Detailed power settings for SmartAudio V2.1+.
    pub power_settings: Option<PowerSettings>,
}

/// Response to a `SetPowerCommand`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetPowerResponse {
    /// The new power. For V1/V2 this is a level index, for V2.1 it's in dBm.
    power: u8,
}

impl SmartAudioResponse for SetPowerResponse {
    fn try_from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, SmartAudioError> {
        Self::try_from(raw_frame)
    }
}

/// Response to a `SetChannelCommand`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetChannelResponse {
    /// The new channel index (0-39).
    channel: u8,
}

impl SmartAudioResponse for SetChannelResponse {
    fn try_from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, SmartAudioError> {
        Self::try_from(raw_frame)
    }
}

/// Response to a `SetFrequencyCommand`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetFrequencyResponse {
    /// The new frequency in MHz.
    frequency: u16,
}

impl SmartAudioResponse for SetFrequencyResponse {
    fn try_from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, SmartAudioError> {
        Self::try_from(raw_frame)
    }
}

/// Response to a `SetModeCommand`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetModeResponse {
    /// In-range pit mode is active.
    pub pitmode_in_range_active: bool,
    /// Out-of-range pit mode is active.
    pub pitmode_out_range_active: bool,
    /// Pit mode is running.
    pub pitmode_enabled: bool,
    /// VTX is unlocked.
    pub unlocked: bool,
}

impl SmartAudioResponse for SetModeResponse {
    fn try_from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, SmartAudioError> {
        Self::try_from(raw_frame)
    }
}

impl SmartAudioResponse for Settings {
    fn try_from_raw_frame(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, SmartAudioError> {
        Self::try_from(raw_frame)
    }
}

impl TryFrom<&RawSmartAudioFrame<'_>> for SetPowerResponse {
    type Error = SmartAudioError;

    fn try_from(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, Self::Error> {
        let [power, ..] = raw_frame.payload() else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };
        Ok(Self { power: *power })
    }
}

impl TryFrom<&RawSmartAudioFrame<'_>> for SetChannelResponse {
    type Error = SmartAudioError;

    fn try_from(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, Self::Error> {
        let [channel, ..] = raw_frame.payload() else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };
        Ok(Self { channel: *channel })
    }
}

impl TryFrom<&RawSmartAudioFrame<'_>> for SetFrequencyResponse {
    type Error = SmartAudioError;

    fn try_from(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, Self::Error> {
        let [f0, f1, ..] = raw_frame.payload() else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };
        Ok(Self {
            frequency: u16::from_be_bytes([*f0, *f1]),
        })
    }
}

impl TryFrom<&RawSmartAudioFrame<'_>> for SetModeResponse {
    type Error = SmartAudioError;

    fn try_from(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, Self::Error> {
        let [mode, ..] = raw_frame.payload() else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };
        Ok(Self {
            pitmode_in_range_active: mode & mode_flags::PITMODE_IN_RANGE != 0,
            pitmode_out_range_active: mode & mode_flags::PITMODE_OUT_RANGE != 0,
            pitmode_enabled: mode & mode_flags::PITMODE_ENABLED != 0,
            unlocked: mode & mode_flags::UNLOCKED != 0,
        })
    }
}

impl TryFrom<&RawSmartAudioFrame<'_>> for Settings {
    type Error = SmartAudioError;

    fn try_from(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, Self::Error> {
        let [channel, power_level, mode, f0, f1, rest @ ..] = raw_frame.payload() else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };

        let version = Version::from(raw_frame.command());

        // unpack mode
        let pitmode_enabled = mode & get_settings_flags::PITMODE_ENABLED != 0;
        let pitmode_in_range_active = mode & get_settings_flags::PITMODE_IN_RANGE != 0;
        let pitmode_out_range_active = mode & get_settings_flags::PITMODE_OUT_RANGE != 0;
        let unlocked = mode & get_settings_flags::UNLOCKED != 0;
        let user_frequency_mode = mode & get_settings_flags::USER_FREQUENCY != 0;

        let frequency = u16::from_be_bytes([*f0, *f1]);

        let power_settings = if version == Version::V2_1 {
            let [current_power, num_power_levels, dbm_level_1, dbm_level_2, dbm_level_3, dbm_level_4, ..] =
                rest
            else {
                return Err(SmartAudioError::InvalidPayloadLength);
            };
            Some(PowerSettings {
                current_power: *current_power,
                num_power_levels: *num_power_levels,
                dbm_level_1: *dbm_level_1,
                dbm_level_2: *dbm_level_2,
                dbm_level_3: *dbm_level_3,
                dbm_level_4: *dbm_level_4,
            })
        } else {
            None
        };

        Ok(Self {
            version,
            channel: *channel,
            power_level: *power_level,
            frequency,
            unlocked,
            user_frequency_mode,
            pitmode_enabled,
            pitmode_in_range_active,
            pitmode_out_range_active,
            power_settings,
        })
    }
}

/// A parsed response from the VTX.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Response {
    /// Current VTX settings (`GetSettingsCommand` response).
    GetSettings(Settings),
    /// Power setting confirmation (`SetPowerCommand` response).
    SetPower(SetPowerResponse),
    /// Channel setting confirmation (`SetChannelCommand` response).
    SetChannel(SetChannelResponse),
    /// Frequency setting confirmation (`SetFrequencyCommand` response).
    SetFrequency(SetFrequencyResponse),
    /// Mode setting confirmation (`SetModeCommand` response).
    SetMode(SetModeResponse),
    /// An unknown or unsupported response.
    Unknown(u8),
}

impl Response {
    pub fn parse(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, SmartAudioError> {
        let cmd = raw_frame.command();
        match cmd {
            resp::GET_SETTINGS_V1_0 | resp::GET_SETTINGS_V2_0 | resp::GET_SETTINGS_V2_1 => {
                Ok(Self::GetSettings(Settings::try_from(raw_frame)?))
            }
            resp::SET_POWER => Ok(Self::SetPower(SetPowerResponse::try_from(raw_frame)?)),
            resp::SET_CHANNEL => Ok(Self::SetChannel(SetChannelResponse::try_from(raw_frame)?)),
            resp::SET_FREQUENCY => Ok(Self::SetFrequency(SetFrequencyResponse::try_from(
                raw_frame,
            )?)),
            resp::SET_MODE => Ok(Self::SetMode(SetModeResponse::try_from(raw_frame)?)),
            _ => Err(SmartAudioError::InvalidHeader),
        }
    }
}

impl SmartAudioParser {
    pub fn parse_bytes<'a, 'b>(&'a mut self, buffer: &'b [u8]) -> ResponseIterator<'a, 'b> {
        self.iter_responses(buffer)
    }

    pub fn iter_responses<'a, 'b>(&'a mut self, buffer: &'b [u8]) -> ResponseIterator<'a, 'b> {
        ResponseIterator {
            parser: self,
            buffer,
            position: 0,
        }
    }
}

pub struct ResponseIterator<'a, 'b> {
    parser: &'a mut SmartAudioParser,
    buffer: &'b [u8],
    position: usize,
}

impl Iterator for ResponseIterator<'_, '_> {
    type Item = Result<Response, SmartAudioError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(byte) = self.buffer.get(self.position).copied() {
            self.position += 1;

            match self.parser.push_byte(byte) {
                Ok(Some(response)) => return Some(Ok(response)),
                Ok(None) => (),
                Err(e) => return Some(Err(e)),
            }
        }
        None
    }
}

impl SmartAudioParser {
    pub fn push_byte(&mut self, byte: u8) -> Result<Option<Response>, SmartAudioError> {
        let Some(raw_packet) = self.push_byte_raw(byte)? else {
            return Ok(None);
        };
        Response::parse(&raw_packet).map(Some)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::parser::RawSmartAudioFrame;
    use crate::parser::SmartAudioError;
    use std::vec::Vec;

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

    #[test]
    fn test_push_byte_get_settings() {
        let raw: [u8; 10] = [0xAA, 0x55, 0x09, 0x06, 0x01, 0x00, 0x1A, 0x16, 0xE9, 0x0A];
        let mut parser = SmartAudioParser::default();

        for byte in raw.iter().take(raw.len() - 1) {
            assert!(matches!(parser.push_byte(*byte), Ok(None)));
        }

        let result = parser.push_byte(raw[raw.len() - 1]);
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
        assert!(matches!(result, Ok(Some(Response::GetSettings(actual))) if actual == expected));
    }

    #[test]
    fn test_push_byte_invalid_crc() {
        // Same as test_get_settings_v2_0_parsing, but with last byte (CRC) modified
        let raw: [u8; 10] = [0xAA, 0x55, 0x09, 0x06, 0x01, 0x00, 0x1A, 0x16, 0xE9, 0x0B]; // 0x0A is correct CRC
        let mut parser = SmartAudioParser::default();

        for byte in raw.iter().take(raw.len() - 1) {
            assert!(matches!(parser.push_byte(*byte), Ok(None)));
        }

        let result = parser.push_byte(raw[raw.len() - 1]);
        assert!(matches!(result, Err(SmartAudioError::InvalidCrc { .. })));
    }

    #[test]
    fn test_iterator() {
        let raw: [u8; 72] = [
            0xAA, 0x55, 0x01, 0x06, 0x00, 0x00, 0x01, 0x16, 0xE9, 0x4D, // frame0
            0xAA, 0x55, 0x09, 0x06, 0x01, 0x00, 0x1A, 0x16, 0xE9, 0x0A, // frame1
            0xAA, 0x55, 0x11, 0x0C, 0x00, 0x00, 0x00, 0x16, 0xE9, 0x0E, 0x03, 0x00, 0x0E, 0x14,
            0x1A, 0x01, // frame2
            0xAA, 0x55, 0x02, 0x03, 0x00, 0x01, 0x0F, // frame3
            0xAA, 0x55, 0x02, 0x03, 0x0E, 0x01, 0x6D, // frame4
            0xAA, 0x55, 0x03, 0x03, 0x00, 0x01, 0x4A, // frame5
            0xAA, 0x55, 0x04, 0x04, 0x16, 0xE9, 0x01, 0xF8, // frame6
            0xAA, 0x55, 0x05, 0x03, 0x0A, 0x01, 0x4F, // frame7
        ];

        let frame0 = Settings {
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
        let frame1 = Settings {
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
        let frame2 = Settings {
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
        let frame3 = SetPowerResponse { power: 0 };
        let frame4 = SetPowerResponse { power: 14 };
        let frame5 = SetChannelResponse { channel: 0 };
        let frame6 = SetFrequencyResponse { frequency: 5865 };
        let frame7 = SetModeResponse {
            pitmode_in_range_active: false,
            pitmode_out_range_active: true,
            pitmode_enabled: false,
            unlocked: true,
        };
        let mut parser = SmartAudioParser::new();
        let responses: Vec<_> = parser
            .iter_responses(&raw)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(responses.len(), 8);

        assert!(matches!(&responses[0], Response::GetSettings(actual) if actual == &frame0));
        assert!(matches!(&responses[1], Response::GetSettings(actual) if actual == &frame1));
        assert!(matches!(&responses[2], Response::GetSettings(actual) if actual == &frame2));
        assert!(matches!(&responses[3], Response::SetPower(actual) if actual == &frame3));
        assert!(matches!(&responses[4], Response::SetPower(actual) if actual == &frame4));
        assert!(matches!(&responses[5], Response::SetChannel(actual) if actual == &frame5));
        assert!(matches!(&responses[6], Response::SetFrequency(actual) if actual == &frame6));
        assert!(matches!(&responses[7], Response::SetMode(actual) if actual == &frame7));
    }

    #[test]
    fn test_parse_bytes_multiple_frames() {
        let raw: [u8; 21] = [
            0xAA, 0x55, 0x02, 0x03, 0x00, 0x01, 0x0F, // set power
            0xAA, 0x55, 0x03, 0x03, 0x00, 0x01, 0x4A, // set channel
            0xAA, 0x55, 0x05, 0x03, 0x0A, 0x01, 0x4F, // set mode
        ];
        let mut parser = SmartAudioParser::new();
        let responses: Vec<_> = parser
            .parse_bytes(&raw)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(responses.len(), 3);
        assert!(matches!(
            &responses[0],
            Response::SetPower(SetPowerResponse { power: 0 })
        ));
        assert!(matches!(
            &responses[1],
            Response::SetChannel(SetChannelResponse { channel: 0 })
        ));
        assert!(matches!(
            &responses[2],
            Response::SetMode(SetModeResponse {
                pitmode_in_range_active: false,
                pitmode_out_range_active: true,
                pitmode_enabled: false,
                unlocked: true
            })
        ));
    }

    #[test]
    fn test_parse_bytes_returns_error_on_invalid_crc() {
        let raw: [u8; 17] = [
            0xAA, 0x55, 0x02, 0x03, 0x00, 0x01, 0x0F, // valid frame
            0xAA, 0x55, 0x09, 0x06, 0x01, 0x00, 0x1A, 0x16, 0xE9, 0x0B, // invalid crc
        ];
        let mut parser = SmartAudioParser::new();
        let mut responses = parser.parse_bytes(&raw);

        let first = responses.next().unwrap().unwrap();
        assert!(matches!(
            first,
            Response::SetPower(SetPowerResponse { power: 0 })
        ));

        let second = responses.next().unwrap();
        assert!(matches!(second, Err(SmartAudioError::InvalidCrc { .. })));
    }
}
