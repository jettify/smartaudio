use crate::constants::get_settings_flags;
use crate::constants::response as resp;
use crate::constants::set_mode_response_flags as mode_flags;
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

// Response length includes CRC; settings use five base bytes and two power metadata bytes.
const MAX_POWER_LEVELS: usize = crate::constants::MAX_RESPONSE_LENGTH - 8;

/// Detailed power settings, included in `GetSettings` response for SmartAudio V2.1+.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PowerSettings {
    /// The currently selected power in dBm.
    pub current_power: u8,
    num_power_levels: u8,
    dbm_levels: [u8; MAX_POWER_LEVELS],
}

impl PowerSettings {
    /// The reported number of power levels, excluding the initial pit-mode entry.
    pub fn num_power_levels(&self) -> u8 {
        self.num_power_levels
    }

    /// All reported power levels in dBm, including the initial zero-dBm pit-mode entry.
    /// The slice contains `num_power_levels() + 1` entries.
    pub fn dbm_levels(&self) -> &[u8] {
        self.dbm_levels
            .get(..usize::from(self.num_power_levels) + 1)
            .unwrap_or_default()
    }
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
    /// The confirmed power: a DAC value for V1, a level index for V2,
    /// or a dBm value for V2.1.
    pub power: u8,
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
    pub channel: u8,
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
    pub frequency: u16,
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
        if raw_frame.command() != resp::SET_POWER {
            return Err(SmartAudioError::UnknownCommand(raw_frame.command()));
        }
        let [power, _reserved] = raw_frame.payload() else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };
        Ok(Self { power: *power })
    }
}

impl TryFrom<&RawSmartAudioFrame<'_>> for SetChannelResponse {
    type Error = SmartAudioError;

    fn try_from(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, Self::Error> {
        if raw_frame.command() != resp::SET_CHANNEL {
            return Err(SmartAudioError::UnknownCommand(raw_frame.command()));
        }
        let [channel, _reserved] = raw_frame.payload() else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };
        Ok(Self { channel: *channel })
    }
}

impl TryFrom<&RawSmartAudioFrame<'_>> for SetFrequencyResponse {
    type Error = SmartAudioError;

    fn try_from(raw_frame: &RawSmartAudioFrame<'_>) -> Result<Self, Self::Error> {
        if raw_frame.command() != resp::SET_FREQUENCY {
            return Err(SmartAudioError::UnknownCommand(raw_frame.command()));
        }
        let [f0, f1, _reserved] = raw_frame.payload() else {
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
        if raw_frame.command() != resp::SET_MODE {
            return Err(SmartAudioError::UnknownCommand(raw_frame.command()));
        }
        let [mode, _reserved] = raw_frame.payload() else {
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
        let version = Version::from(raw_frame.command());
        if version == Version::Unknown {
            return Err(SmartAudioError::UnknownCommand(raw_frame.command()));
        }
        let [channel, power_level, mode, f0, f1, rest @ ..] = raw_frame.payload() else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };

        // unpack mode
        let pitmode_enabled = mode & get_settings_flags::PITMODE_ENABLED != 0;
        let pitmode_in_range_active = mode & get_settings_flags::PITMODE_IN_RANGE != 0;
        let pitmode_out_range_active = mode & get_settings_flags::PITMODE_OUT_RANGE != 0;
        let unlocked = mode & get_settings_flags::UNLOCKED != 0;
        let user_frequency_mode = mode & get_settings_flags::USER_FREQUENCY != 0;

        let frequency = u16::from_be_bytes([*f0, *f1]);

        let power_settings = if version == Version::V2_1 {
            let [current_power, num_power_levels, levels @ ..] = rest else {
                return Err(SmartAudioError::InvalidPayloadLength);
            };
            let expected_levels = usize::from(*num_power_levels) + 1;
            if levels.len() != expected_levels {
                return Err(SmartAudioError::InvalidPayloadLength);
            }
            let mut dbm_levels = [0; MAX_POWER_LEVELS];
            dbm_levels
                .get_mut(..expected_levels)
                .ok_or(SmartAudioError::InvalidPayloadLength)?
                .copy_from_slice(levels);
            Some(PowerSettings {
                current_power: *current_power,
                num_power_levels: *num_power_levels,
                dbm_levels,
            })
        } else {
            if !rest.is_empty() {
                return Err(SmartAudioError::InvalidPayloadLength);
            }
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
}

impl Response {
    /// Decode a validated response frame. Unsupported commands return `UnknownCommand`;
    /// known commands with malformed payloads return `InvalidPayloadLength`.
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
            _ => Err(SmartAudioError::UnknownCommand(cmd)),
        }
    }
}

impl SmartAudioParser {
    /// Alias for [`Self::iter_responses`]. Parsing is lazy; unconsumed bytes are not fed.
    pub fn parse_bytes<'a, 'b>(&'a mut self, input: &'b [u8]) -> ResponseIterator<'a, 'b> {
        self.iter_responses(input)
    }

    /// Lazily parse a chunk, retaining an incomplete frame for the next chunk.
    /// Errors are yielded as items; advancing the iterator continues parsing.
    /// Dropping the iterator leaves its remaining input bytes unconsumed.
    pub fn iter_responses<'a, 'b>(&'a mut self, input: &'b [u8]) -> ResponseIterator<'a, 'b> {
        ResponseIterator {
            parser: self,
            input,
        }
    }
}

pub struct ResponseIterator<'a, 'b> {
    parser: &'a mut SmartAudioParser,
    input: &'b [u8],
}

impl Iterator for ResponseIterator<'_, '_> {
    type Item = Result<Response, SmartAudioError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.next_frame(&mut self.input) {
            Ok(Some(frame)) => Some(Response::parse(&frame)),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::parser::RawSmartAudioFrame;
    use crate::parser::SmartAudioError;
    use std::vec::Vec;

    // Build structurally valid response frames for typed-decoding tests.
    fn response_bytes(command: u8, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::from([0xAA, 0x55, command, (payload.len() + 1) as u8]);
        bytes.extend_from_slice(payload);
        let crc = crate::framing::crc8_dvb_s2(&bytes[2..]);
        bytes.push(crc);
        bytes
    }

    fn assert_conversion_command<T: SmartAudioResponse>(
        frame: &RawSmartAudioFrame<'_>,
        direct: Result<T, SmartAudioError>,
        expected_command: bool,
    ) {
        let direct = direct.map(|_| ());
        assert_eq!(direct, T::try_from_raw_frame(frame).map(|_| ()));
        if expected_command {
            assert_eq!(direct, Ok(()));
        } else {
            assert_eq!(
                direct,
                Err(SmartAudioError::UnknownCommand(frame.command()))
            );
        }
    }

    #[test]
    fn dropping_iterator_does_not_consume_remaining_input() {
        let valid = response_bytes(0x03, &[0, 1]);
        let mut chunk = valid.clone();
        chunk.extend_from_slice(&[0xAA, 0x55]);
        let mut parser = SmartAudioParser::new();
        assert!(parser.iter_responses(&chunk).next().unwrap().is_ok());
        // The unconsumed trailing header must not become a buffered partial frame.
        assert!(parser.iter_responses(&valid).next().unwrap().is_ok());

        // Exhausting the iterator does retain the trailing header.
        assert_eq!(parser.iter_responses(&chunk).count(), 1);
        assert!(parser
            .iter_responses(valid.get(2..).unwrap())
            .next()
            .unwrap()
            .is_ok());
    }

    #[test]
    fn reports_unknown_and_malformed_responses_then_recovers_across_chunks() {
        let mut stream = response_bytes(0xFF, &[]);
        stream.extend(response_bytes(0x03, &[0]));
        stream.extend(response_bytes(0x03, &[0, 1]));

        for split in 0..=stream.len() {
            let mut parser = SmartAudioParser::new();
            let mut results: Vec<_> = parser.iter_responses(&stream[..split]).collect();
            results.extend(parser.iter_responses(&stream[split..]));
            assert_eq!(results.len(), 3);
            assert!(matches!(
                results[0],
                Err(SmartAudioError::UnknownCommand(0xFF))
            ));
            assert!(matches!(
                results[1],
                Err(SmartAudioError::InvalidPayloadLength)
            ));
            assert!(matches!(
                results[2],
                Ok(Response::SetChannel(SetChannelResponse { channel: 0 }))
            ));
        }
    }

    #[test]
    fn set_mode_response_reports_running_pitmode() {
        let bytes = response_bytes(0x05, &[0x04, 1]);
        let mut parser = SmartAudioParser::new();
        let response = parser.iter_responses(&bytes).next().unwrap().unwrap();
        assert!(matches!(
            response,
            Response::SetMode(SetModeResponse {
                pitmode_enabled: true,
                pitmode_in_range_active: false,
                pitmode_out_range_active: false,
                unlocked: false,
            })
        ));
    }

    #[test]
    fn typed_conversions_validate_response_commands() {
        for command in [0x01, 0x09, 0x11, 0x02, 0x03, 0x04, 0x05, 0xFF] {
            let payload_len = match command {
                0x02 | 0x03 | 0x05 => 2,
                0x04 => 3,
                0x11 => 8,
                _ => 5,
            };
            let bytes = response_bytes(command, &Vec::from_iter((0..payload_len).map(|_| 0)));
            let frame = RawSmartAudioFrame::try_from(bytes.as_slice()).unwrap();
            assert_conversion_command(&frame, SetPowerResponse::try_from(&frame), command == 0x02);
            assert_conversion_command(
                &frame,
                SetChannelResponse::try_from(&frame),
                command == 0x03,
            );
            assert_conversion_command(
                &frame,
                SetFrequencyResponse::try_from(&frame),
                command == 0x04,
            );
            assert_conversion_command(&frame, SetModeResponse::try_from(&frame), command == 0x05);
            assert_conversion_command(
                &frame,
                Settings::try_from(&frame),
                matches!(command, 0x01 | 0x09 | 0x11),
            );
        }
    }

    #[test]
    fn typed_conversions_validate_fixed_payload_lengths() {
        for (command, expected_len) in [
            (0x01, 5),
            (0x09, 5),
            (0x02, 2),
            (0x03, 2),
            (0x04, 3),
            (0x05, 2),
        ] {
            for payload_len in 0..=8 {
                let payload = Vec::from_iter((0..payload_len).map(|_| 0));
                let bytes = response_bytes(command, &payload);
                let frame = RawSmartAudioFrame::try_from(bytes.as_slice()).unwrap();
                let result = match command {
                    0x01 | 0x09 => Settings::try_from(&frame).map(|_| ()),
                    0x02 => SetPowerResponse::try_from(&frame).map(|_| ()),
                    0x03 => SetChannelResponse::try_from(&frame).map(|_| ()),
                    0x04 => SetFrequencyResponse::try_from(&frame).map(|_| ()),
                    0x05 => SetModeResponse::try_from(&frame).map(|_| ()),
                    _ => unreachable!(),
                };
                if payload_len == expected_len {
                    assert_eq!(result, Ok(()));
                } else {
                    assert_eq!(result, Err(SmartAudioError::InvalidPayloadLength));
                }
            }
        }
    }

    #[test]
    fn test_get_settings_v1_0_parsing() {
        let raw: [u8; 10] = [0xAA, 0x55, 0x01, 0x06, 0x00, 0x00, 0x01, 0x16, 0xE9, 0x4D];
        let frame = RawSmartAudioFrame::try_from(raw.as_slice()).unwrap();
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
        let frame = RawSmartAudioFrame::try_from(raw.as_slice()).unwrap();
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
        let frame = RawSmartAudioFrame::try_from(raw.as_slice()).unwrap();
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
                dbm_levels: {
                    let mut levels = [0; MAX_POWER_LEVELS];
                    levels[..4].copy_from_slice(&[0, 14, 20, 26]);
                    levels
                },
            }),
        };
        assert!(matches!(packet, Response::GetSettings(actual) if actual == expected));
    }

    #[test]
    fn parses_variable_length_v2_1_power_tables() {
        let cases: &[(&[u8], u8, &[u8])] = &[
            (
                &[0xAA, 0x55, 0x11, 0x09, 0, 0, 0, 0x16, 0xE9, 14, 0, 0, 0x32],
                0,
                &[0],
            ),
            (
                &[
                    0xAA, 0x55, 0x11, 0x0B, 0, 0, 0, 0x16, 0xE9, 14, 2, 0, 14, 20, 0x3F,
                ],
                2,
                &[0, 14, 20],
            ),
            (
                &[
                    0xAA, 0x55, 0x11, 0x0D, 0, 0, 0, 0x16, 0xE9, 14, 4, 0, 14, 20, 26, 30, 0x64,
                ],
                4,
                &[0, 14, 20, 26, 30],
            ),
            (
                &[
                    0xAA, 0x55, 0x11, 0x1C, 0, 0, 0, 0x16, 0xE9, 14, 19, 0, 10, 11, 12, 13, 14, 15,
                    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 0xCB,
                ],
                19,
                &[
                    0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
                ],
            ),
        ];

        for &(raw, expected_count, expected_levels) in cases {
            let mut parser = SmartAudioParser::new();
            let mut responses = parser.iter_responses(raw);
            let Response::GetSettings(settings) = responses.next().unwrap().unwrap() else {
                panic!("expected settings response");
            };
            let power = settings.power_settings.unwrap();
            assert_eq!(power.current_power, 14);
            assert_eq!(power.num_power_levels(), expected_count);
            assert_eq!(power.dbm_levels(), expected_levels);
            assert!(responses.next().is_none());
        }
    }

    #[test]
    fn rejects_v2_1_power_table_count_mismatches() {
        // Each frame has a valid CRC, but its table does not match count + 1.
        let cases: &[&[u8]] = &[
            &[
                0xAA, 0x55, 0x11, 0x0B, 0, 0, 0, 0x16, 0xE9, 14, 3, 0, 14, 20, 0x7A,
            ],
            &[
                0xAA, 0x55, 0x11, 0x0B, 0, 0, 0, 0x16, 0xE9, 14, 1, 0, 14, 20, 0xF0,
            ],
            &[
                0xAA, 0x55, 0x11, 0x09, 0, 0, 0, 0x16, 0xE9, 14, 255, 0, 0x4A,
            ],
            &[0xAA, 0x55, 0x11, 0x08, 0, 0, 0, 0x16, 0xE9, 14, 0, 0xF6],
        ];

        for &raw in cases {
            let mut parser = SmartAudioParser::new();
            let mut responses = parser.iter_responses(raw);
            assert!(matches!(
                responses.next(),
                Some(Err(SmartAudioError::InvalidPayloadLength))
            ));
            assert!(responses.next().is_none());
        }
    }

    #[test]
    fn test_set_power_response_parsing() {
        let raw_v20: [u8; 7] = [0xAA, 0x55, 0x02, 0x03, 0x00, 0x01, 0x0F];
        let frame_v20 = RawSmartAudioFrame::try_from(raw_v20.as_slice()).unwrap();
        let packet_v20 = Response::parse(&frame_v20).unwrap();
        let expected = SetPowerResponse { power: 0 };
        assert!(matches!(packet_v20, Response::SetPower(actual) if actual == expected));

        let raw_v21: [u8; 7] = [0xAA, 0x55, 0x02, 0x03, 0x0E, 0x01, 0x6D];
        let frame_v21 = RawSmartAudioFrame::try_from(raw_v21.as_slice()).unwrap();
        let packet_v21 = Response::parse(&frame_v21).unwrap();
        let expected = SetPowerResponse { power: 14 };
        assert!(matches!(packet_v21, Response::SetPower(actual) if actual == expected));
    }

    #[test]
    fn test_set_channel_response_parsing() {
        let raw: [u8; 7] = [0xAA, 0x55, 0x03, 0x03, 0x00, 0x01, 0x4A];
        let frame = RawSmartAudioFrame::try_from(raw.as_slice()).unwrap();
        let packet = Response::parse(&frame).unwrap();
        let expected = SetChannelResponse { channel: 0 };
        assert!(matches!(packet, Response::SetChannel(actual) if actual == expected));
    }

    #[test]
    fn test_set_frequency_response_parsing() {
        let raw: [u8; 8] = [0xAA, 0x55, 0x04, 0x04, 0x16, 0xE9, 0x01, 0xF8];
        let frame = RawSmartAudioFrame::try_from(raw.as_slice()).unwrap();
        let packet = Response::parse(&frame).unwrap();
        let expected = SetFrequencyResponse { frequency: 5865 };
        assert!(matches!(packet, Response::SetFrequency(actual) if actual == expected));
    }

    #[test]
    fn test_set_mode_response_parsing() {
        let raw: [u8; 7] = [0xAA, 0x55, 0x05, 0x03, 0x0A, 0x01, 0x4F];
        let frame = RawSmartAudioFrame::try_from(raw.as_slice()).unwrap();
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
    fn parses_settings_across_chunks() {
        let raw: [u8; 10] = [0xAA, 0x55, 0x09, 0x06, 0x01, 0x00, 0x1A, 0x16, 0xE9, 0x0A];
        let mut parser = SmartAudioParser::default();

        let (prefix, suffix) = raw.split_at(raw.len() - 1);
        assert!(parser.iter_responses(prefix).next().is_none());
        let result = parser.iter_responses(suffix).next().unwrap();
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
        assert!(matches!(result, Ok(Response::GetSettings(actual)) if actual == expected));
    }

    #[test]
    fn reports_invalid_crc_across_chunks() {
        // Same as test_get_settings_v2_0_parsing, but with last byte (CRC) modified
        let raw: [u8; 10] = [0xAA, 0x55, 0x09, 0x06, 0x01, 0x00, 0x1A, 0x16, 0xE9, 0x0B]; // 0x0A is correct CRC
        let mut parser = SmartAudioParser::default();

        let (prefix, suffix) = raw.split_at(raw.len() - 1);
        assert!(parser.iter_responses(prefix).next().is_none());
        let result = parser.iter_responses(suffix).next().unwrap();
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
                dbm_levels: {
                    let mut levels = [0; MAX_POWER_LEVELS];
                    levels[..4].copy_from_slice(&[0, 14, 20, 26]);
                    levels
                },
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
