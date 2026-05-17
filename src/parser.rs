use crate::constants;

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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SmartAudioError {
    BufferTooSmall(usize),
    InvalidCrc { calculated_crc: u8, frame_crc: u8 },
    InvalidHeader,
    UnknownCommand(u8),
    InvalidPayloadLength,
    UnexpectedDataForState(State, u8),
}

pub fn frame_payload(
    buffer: &mut [u8],
    command: u8,
    payload: &[u8],
) -> Result<usize, SmartAudioError> {
    let payload_size = payload.len();
    let frame_len = 2 + 1 + 1 + payload_size + 1;
    if buffer.len() < frame_len {
        return Err(SmartAudioError::BufferTooSmall(buffer.len()));
    }
    let frame = buffer
        .get_mut(..frame_len)
        .ok_or(SmartAudioError::BufferTooSmall(frame_len))?;
    let (head, crc_slot) = frame.split_at_mut(frame_len - 1);
    let (header, body) = head.split_at_mut(4);
    let [h1, h2, cmd, len] = header else {
        return Err(SmartAudioError::InvalidPayloadLength);
    };
    *h1 = constants::HEADER_BYTE_1;
    *h2 = constants::HEADER_BYTE_2;
    *cmd = command;
    *len = payload_size as u8;
    body.copy_from_slice(payload);

    let crc = crc8_dvb_s2(head);
    let [crc_byte] = crc_slot else {
        return Err(SmartAudioError::InvalidPayloadLength);
    };
    *crc_byte = crc;
    Ok(frame_len)
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
    command: u8,
    payload: &'a [u8],
    crc: u8,
}

impl<'a> RawSmartAudioFrame<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Result<Self, SmartAudioError> {
        let Some((&command, tail)) = bytes.get(2).zip(bytes.get(3..)) else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };
        let Some((&crc, without_crc)) = bytes.last().zip(bytes.get(4..bytes.len() - 1)) else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };
        let payload_size = tail
            .first()
            .copied()
            .ok_or(SmartAudioError::InvalidPayloadLength)? as usize;
        if payload_size == 0 || without_crc.len() + 1 != payload_size {
            return Err(SmartAudioError::InvalidPayloadLength);
        }
        Ok(Self {
            bytes,
            command,
            payload: without_crc,
            crc,
        })
    }

    pub fn command(&self) -> u8 {
        self.command
    }

    pub fn payload(&self) -> &[u8] {
        self.payload
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Returns the CRC check byte of the frame.
    pub fn crc(&self) -> u8 {
        self.crc
    }
}

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

    pub fn reset(&mut self) {
        self.position = 0;
        self.state = State::AwaitingHeader1;
    }

    pub fn push_byte_raw(
        &mut self,
        byte: u8,
    ) -> Result<Option<RawSmartAudioFrame<'_>>, SmartAudioError> {
        let write_next = |this: &mut Self, value: u8| -> Result<(), SmartAudioError> {
            let slot = this
                .buffer
                .get_mut(this.position)
                .ok_or(SmartAudioError::InvalidPayloadLength)?;
            *slot = value;
            Ok(())
        };

        match self.state {
            State::AwaitingHeader1 if byte == constants::HEADER_BYTE_1 => {
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
                if (constants::MIN_PAYLOAD_SIZE..constants::MAX_PAYLOAD_SIZE)
                    .contains(&(byte as usize)) =>
            {
                self.position += 1;
                write_next(self, byte)?;
                self.state = State::Reading(byte as usize);
                Ok(None)
            }
            State::Reading(n) => {
                self.position += 1;
                write_next(self, byte)?;
                if self.position == n + 3 {
                    let start = 0;
                    let end = self.position + 1;

                    let crc_data = self
                        .buffer
                        .get(2..end - 1)
                        .ok_or(SmartAudioError::InvalidPayloadLength)?;
                    let calculated_crc = crc8_dvb_s2(crc_data);

                    let frame_crc = *self
                        .buffer
                        .get(self.position)
                        .ok_or(SmartAudioError::InvalidPayloadLength)?;
                    if frame_crc != calculated_crc {
                        return Err(SmartAudioError::InvalidCrc {
                            frame_crc,
                            calculated_crc,
                        });
                    }
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
