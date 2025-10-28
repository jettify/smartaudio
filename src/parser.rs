// Constants for SmartAudio protocol
const HEADER_BYTE_1: u8 = 0xAA;
const HEADER_BYTE_2: u8 = 0x55;
const MAX_FRAME_SIZE: usize = 32;
const MAX_PAYLOAD_SIZE: usize = 28;
const MIN_PAYLOAD_SIZE: usize = 3;

fn crc8_dvb_s2(data: &[u8]) -> u8 {
    let mut crc = 0;
    for byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc = (crc << 1) ^ 0xD5;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartAudioError {
    BufferTooSmall(usize),
    InvalidCrc { calculated_crc: u8, frame_crc: u8 },
    InvalidHeader,
    UnknownCommand(u8),
    InvalidPayloadLength,
    UnexpetedDataForState(State, u8),
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawSmartAudioFrame<'a> {
    bytes: &'a [u8],
}

impl<'a> RawSmartAudioFrame<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() >= 4 {
            Some(Self { bytes })
        } else {
            None
        }
    }

    pub fn commnand(&self) -> u8 {
        self.bytes[2]
    }

    pub fn payload(&self) -> &[u8] {
        &self.bytes[4..self.bytes.len() - 1]
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Returns the CRC check byte of the frame.
    #[expect(clippy::missing_panics_doc, reason = "infallible")]
    pub fn crc(&self) -> u8 {
        *self.bytes.last().expect("infallible due to length check")
    }
}

#[derive(Debug)]
pub struct SmartAudioParser {
    buffer: [u8; MAX_FRAME_SIZE],
    state: State,
    position: usize,
}

impl SmartAudioParser {
    pub fn new() -> Self {
        Self {
            buffer: [0; MAX_FRAME_SIZE],
            state: State::AwaitingHeader1,
            position: 0,
        }
    }

    pub fn reset(&mut self) {
        self.position = 0;
        self.state = State::AwaitingHeader1;
    }

    pub fn push_byte_raw(
        &mut self,
        byte: u8,
    ) -> Result<Option<RawSmartAudioFrame<'_>>, SmartAudioError> {
        match self.state {
            State::AwaitingHeader1 if byte == HEADER_BYTE_1 => {
                self.position = 0;
                self.buffer[self.position] = byte;
                self.state = State::AwaitingHeader2;
                Ok(None)
            }
            State::AwaitingHeader2 if byte == HEADER_BYTE_2 => {
                self.position += 1;
                self.buffer[self.position] = byte;
                self.state = State::AwaitingCommand;
                Ok(None)
            }
            State::AwaitingCommand => {
                self.position += 1;
                self.buffer[self.position] = byte;
                self.state = State::AwaitingLength;
                Ok(None)
            }
            State::AwaitingLength
                if (MIN_PAYLOAD_SIZE..MAX_PAYLOAD_SIZE).contains(&(byte as usize)) =>
            {
                self.position += 1;
                self.buffer[self.position] = byte;
                self.state = State::Reading(byte as usize);
                Ok(None)
            }
            State::Reading(n) => {
                self.position += 1;
                self.buffer[self.position] = byte;
                if self.position == n + 3 {
                    let start = 0;
                    let end = self.position + 1;

                    let calculated_crc = crc8_dvb_s2(&self.buffer[2..end - 1]);

                    let frame_crc = self.buffer[self.position];
                    if frame_crc != calculated_crc {
                        return Err(SmartAudioError::InvalidCrc {
                            frame_crc,
                            calculated_crc,
                        });
                    }
                    self.reset();
                    let bytes = &self.buffer[start..end];
                    Ok(RawSmartAudioFrame::new(bytes))
                } else {
                    Ok(None)
                }
            }
            _ => {
                let current_state = self.state;
                self.reset();
                Err(SmartAudioError::UnexpetedDataForState(current_state, byte))
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
    fn test_raw_responses_parsing() {
        // Raw data obteainted form protcol specificaton examples
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
