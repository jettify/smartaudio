use crate::constants;

// Preserve the original public paths.
pub use crate::framing::{frame_payload, RawSmartAudioFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SmartAudioError {
    /// The output buffer is too small; contains the required byte count.
    BufferTooSmall(usize),
    /// A response's CRC does not match command, length, and payload.
    InvalidCrc { calculated_crc: u8, frame_crc: u8 },
    /// A complete frame does not start with `0xAA 0x55`.
    InvalidHeader,
    /// An unsupported command, or one that does not match the requested response type.
    UnknownCommand(u8),
    /// Invalid wire length, command-specific payload shape, or encoder payload over 255 bytes.
    InvalidPayloadLength,
    /// A byte cannot continue the current header state; the candidate was discarded.
    UnexpectedDataForState(State, u8),
}

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    #[default]
    AwaitingHeader1,
    AwaitingHeader2,
    AwaitingCommand,
    AwaitingLength,
    Reading(usize),
}

/// Incremental VTX-to-host response parser with a fixed 32-byte buffer.
///
/// Partial frames persist across byte calls and input chunks. Every reported
/// error discards the current candidate; keep feeding bytes to find the next
/// frame. The parser does not track time: call [`Self::reset`] after a timeout
/// or transport interruption to discard a partial frame.
#[derive(Debug)]
pub struct SmartAudioParser {
    buffer: [u8; constants::MAX_FRAME_SIZE],
    state: State,
    position: usize,
}

impl SmartAudioParser {
    pub fn new() -> Self {
        Self {
            buffer: [0; constants::MAX_FRAME_SIZE],
            state: State::AwaitingHeader1,
            position: 0,
        }
    }

    /// Discard a partial frame and resume looking for a header.
    pub fn reset(&mut self) {
        self.position = 0;
        self.state = State::AwaitingHeader1;
    }

    /// Feed one response byte, returning a frame only after header, length, and CRC validation.
    /// Non-header bytes while seeking sync are reported as `UnexpectedDataForState`.
    /// Repeated `0xAA` bytes preserve the candidate header. A returned frame borrows
    /// this parser, so consume it before feeding more bytes.
    pub fn push_byte_raw(
        &mut self,
        byte: u8,
    ) -> Result<Option<RawSmartAudioFrame<'_>>, SmartAudioError> {
        let write_next = |this: &mut Self, value: u8| -> Result<(), SmartAudioError> {
            let Some(slot) = this.buffer.get_mut(this.position) else {
                this.reset();
                return Err(SmartAudioError::InvalidPayloadLength);
            };
            *slot = value;
            Ok(())
        };

        match self.state {
            // A repeated first header byte starts a new candidate without losing sync.
            State::AwaitingHeader1 | State::AwaitingHeader2 if byte == constants::HEADER_BYTE_1 => {
                self.position = 0;
                write_next(self, byte)?;
                self.state = State::AwaitingHeader2;
                Ok(None)
            }
            State::AwaitingHeader2 if byte == constants::HEADER_BYTE_2 => {
                self.position += 1;
                write_next(self, byte)?;
                self.state = State::AwaitingCommand;
                Ok(None)
            }
            State::AwaitingCommand => {
                self.position += 1;
                write_next(self, byte)?;
                self.state = State::AwaitingLength;
                Ok(None)
            }
            State::AwaitingLength
                if (constants::MIN_RESPONSE_LENGTH..=constants::MAX_RESPONSE_LENGTH)
                    .contains(&(byte as usize)) =>
            {
                self.position += 1;
                write_next(self, byte)?;
                self.state = State::Reading(byte as usize);
                Ok(None)
            }
            State::AwaitingLength => {
                self.reset();
                Err(SmartAudioError::InvalidPayloadLength)
            }
            State::Reading(n) => {
                self.position += 1;
                write_next(self, byte)?;
                if self.position + 1 == n + constants::RESPONSE_PREFIX_SIZE {
                    let start = 0;
                    let end = self.position + 1;

                    self.reset();
                    let bytes = self
                        .buffer
                        .get(start..end)
                        .ok_or(SmartAudioError::InvalidPayloadLength)?;
                    Ok(Some(RawSmartAudioFrame::new(bytes)?))
                } else {
                    Ok(None)
                }
            }
            _ => {
                let current_state = self.state;
                self.reset();
                Err(SmartAudioError::UnexpectedDataForState(current_state, byte))
            }
        }
    }
}

impl Default for SmartAudioParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn recovers_after_invalid_response_lengths() {
        for length in [0, 29, 255] {
            let mut parser = SmartAudioParser::new();
            for byte in [0xAA, 0x55, 0x02] {
                assert!(matches!(parser.push_byte_raw(byte), Ok(None)));
            }
            assert_eq!(
                parser.push_byte_raw(length).unwrap_err(),
                SmartAudioError::InvalidPayloadLength
            );
            for byte in [0xAA, 0x55, 0x02, 0x03, 0, 1] {
                assert!(matches!(parser.push_byte_raw(byte), Ok(None)));
            }
            assert_eq!(parser.push_byte_raw(0x0F).unwrap().unwrap().command(), 0x02);
        }
    }

    #[test]
    fn parses_valid_frame_after_overlapping_headers() {
        let valid = [0xAA, 0x55, 0x02, 0x03, 0x00, 0x01, 0x0F];

        for extra_headers in [1, 2, 32] {
            let mut parser = SmartAudioParser::new();
            for _ in 0..extra_headers {
                assert!(matches!(parser.push_byte_raw(0xAA), Ok(None)));
            }
            for &byte in valid.iter().take(valid.len() - 1) {
                assert!(matches!(parser.push_byte_raw(byte), Ok(None)));
            }
            let frame = parser.push_byte_raw(0x0F).unwrap().unwrap();
            assert_eq!(frame.command(), 0x02);
            assert_eq!(frame.payload(), &[0x00, 0x01]);
            assert_eq!(frame.len(), valid.len());
        }
    }

    #[test]
    fn test_parses_valid_frame_after_invalid_crc() {
        let invalid = [0xAA, 0x55, 0x09, 0x06, 0x01, 0x00, 0x1A, 0x16, 0xE9, 0x0B];
        let valid = [0xAA, 0x55, 0x02, 0x03, 0x00, 0x01, 0x0F];
        let mut parser = SmartAudioParser::new();

        for &byte in invalid.iter().take(invalid.len() - 1) {
            assert!(matches!(parser.push_byte_raw(byte), Ok(None)));
        }
        assert!(matches!(
            parser.push_byte_raw(0x0B),
            Err(SmartAudioError::InvalidCrc {
                calculated_crc: 0x0A,
                frame_crc: 0x0B,
            })
        ));

        // Recovery must happen automatically, without an explicit reset.
        for _ in 0..2 {
            for &byte in valid.iter().take(valid.len() - 1) {
                assert!(matches!(parser.push_byte_raw(byte), Ok(None)));
            }
            let frame = parser.push_byte_raw(0x0F).unwrap().unwrap();
            assert_eq!(frame.command(), 0x02);
            assert_eq!(frame.payload(), &[0x00, 0x01]);
            assert_eq!(frame.crc(), 0x0F);
        }
    }

    #[test]
    fn test_raw_responses_parsing() {
        // Raw data obtained from protocol specification examples.
        // https://www.team-blacksheep.com/tbs_smartaudio_rev09.pdf
        let raw_settings_v10: [u8; 10] = [
            0xAA, 0x55, // Header
            0x01, // Version/Command
            0x06, // Length
            0x00, // Channel
            0x00, // Power Level
            0x01, //Operation Mode
            0x16, 0xE9, //Current Frequency 5865
            0x4D, //CRC8
        ];

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

        let raw_settings_v21: [u8; 16] = [
            0xAA, 0x55, // Headers
            0x11, // Version/Command
            0x0C, // Length
            0x00, // Channel
            0x00, // Power Level
            0x00, //Operation Mode
            0x16, 0xE9, //Current Frequency 5865
            0x0E, // current power in dBm)
            0x03, // amount of power levels)
            0x00, // dBm level 1
            0x0E, // dBm level 2
            0x14, // dBm level 3
            0x1A, // dBm level 4)
            0x01, //CRC8
        ];

        let raw_set_power_v20: [u8; 7] = [
            0xAA, 0x55, // Headers
            0x02, // (Command)
            0x03, // (Length)
            0x00, // (Power Level)
            0x01, // (reserved)
            0x0F, // (CRC8)
        ];

        let raw_set_power_v21: [u8; 7] = [
            0xAA, 0x55, // Headers
            0x02, // Command
            0x03, // Length
            0x0E, // Power Level in dBm
            0x01, // reserved
            0x6D, // CRC8
        ];
        let raw_set_channel: [u8; 7] = [
            0xAA, 0x55, // Headers
            0x03, // Command
            0x03, // Length
            0x00, // Channel
            0x01, // Reserved
            0x4A, // CRC8
        ];
        let raw_set_frequency: [u8; 8] = [
            0xAA, 0x55, // Headers
            0x04, // Command
            0x04, // Length
            0x16, 0xE9, //Current Frequency 5865
            0x01, // Reserved
            0xF8, // CRC8
        ];

        let raw_set_mode: [u8; 7] = [
            0xAA, 0x55, // Headers
            0x05, // Command
            0x03, // Length
            0x0A, // Pit mode bits
            0x01, // Reserved
            0x4F, // CRC8
        ];

        let packets = [
            &raw_settings_v10[..],
            &raw_settings_v20[..],
            &raw_settings_v21[..],
            &raw_set_channel[..],
            &raw_set_frequency[..],
            &raw_set_mode[..],
            &raw_set_power_v20[..],
            &raw_set_power_v21[..],
        ];

        for raw_bytes in packets {
            let mut parser = SmartAudioParser::new();
            for b in &raw_bytes[0..raw_bytes.len() - 1] {
                let result = parser.push_byte_raw(*b);
                assert!(matches!(result, Ok(None)));
            }
            let last_index = raw_bytes.len() - 1;
            let p = parser
                .push_byte_raw(raw_bytes[last_index])
                .unwrap()
                .unwrap();
            assert_eq!(p.len(), raw_bytes.len());
            assert_eq!(p.crc(), *raw_bytes.last().unwrap());
        }
    }
}
