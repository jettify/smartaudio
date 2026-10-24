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
}

/// Incremental VTX-to-host response parser with a fixed 32-byte buffer.
///
/// Complete frames borrow the input directly. Only candidates spanning chunks
/// are copied into buffer storage. Errors discard the current candidate; continue
/// with the remaining input to seek the next frame. Call [`Self::reset`] after a
/// timeout or transport interruption to discard an incomplete frame.
#[derive(Debug)]
pub struct SmartAudioParser {
    buffer: [u8; constants::MAX_FRAME_SIZE],
    buffered_len: usize,
}

impl SmartAudioParser {
    pub fn new() -> Self {
        Self {
            buffer: [0; constants::MAX_FRAME_SIZE],
            buffered_len: 0,
        }
    }

    /// Discard a partial frame and resume looking for a header.
    pub fn reset(&mut self) {
        self.buffered_len = 0;
    }

    /// Consume input through the next validated response frame or error.
    ///
    /// Advances `input` on success and error. `Ok(None)` means the input is
    /// exhausted; an incomplete candidate is retained for the next chunk.
    /// Non-header bytes yield `UnexpectedDataForState`; repeated `0xAA` bytes
    /// preserve sync. Invalid lengths consume the four-byte prefix, and invalid
    /// CRCs consume the entire candidate, without rescanning its contents.
    ///
    /// A frame borrows either the input or this parser's buffer. Consume
    /// it before calling this method again. No input reference is retained in
    /// the parser between calls, so exhausted transport buffers can be reused.
    pub fn next_frame<'a, 'input: 'a>(
        &'a mut self,
        input: &mut &'input [u8],
    ) -> Result<Option<RawSmartAudioFrame<'a>>, SmartAudioError> {
        // A fresh first header byte supersedes a lone header from an earlier
        // chunk. Let the direct path borrow the new candidate from this input.
        if self.buffered_len == 1 && input.first() == Some(&constants::HEADER_BYTE_1) {
            self.reset();
        }
        if self.buffered_len > 0 {
            self.complete_buffered_frame(input)
        } else {
            self.parse_input(input)
        }
    }

    fn complete_buffered_frame<'a>(
        &'a mut self,
        input: &mut &[u8],
    ) -> Result<Option<RawSmartAudioFrame<'a>>, SmartAudioError> {
        if self.buffered_len == 1 {
            let Some((&byte, rest)) = input.split_first() else {
                return Ok(None);
            };
            if byte != constants::HEADER_BYTE_2 {
                *input = rest;
                self.reset();
                return Err(SmartAudioError::UnexpectedDataForState(
                    State::AwaitingHeader2,
                    byte,
                ));
            }
        }
        if self.buffered_len < constants::RESPONSE_PREFIX_SIZE {
            self.fill_buffer_to(input, constants::RESPONSE_PREFIX_SIZE)?;
            if self.buffered_len < constants::RESPONSE_PREFIX_SIZE {
                return Ok(None);
            }
        }
        let expected_frame_len = match Self::expected_frame_len(&self.buffer) {
            Ok(length) => length,
            Err(error) => {
                self.reset();
                return Err(error);
            }
        };
        self.fill_buffer_to(input, expected_frame_len)?;
        if self.buffered_len < expected_frame_len {
            return Ok(None);
        }
        // Reset metadata before lending the completed bytes; leave storage intact.
        self.reset();
        let bytes = self
            .buffer
            .get(..expected_frame_len)
            .ok_or(SmartAudioError::InvalidPayloadLength)?;
        RawSmartAudioFrame::try_from(bytes).map(Some)
    }

    fn parse_input<'a, 'input: 'a>(
        &'a mut self,
        input: &mut &'input [u8],
    ) -> Result<Option<RawSmartAudioFrame<'a>>, SmartAudioError> {
        use constants::{HEADER_BYTE_1, HEADER_BYTE_2};

        loop {
            match *input {
                [HEADER_BYTE_1, HEADER_BYTE_1, ..] => {
                    *input = input.split_at(1).1;
                }
                [HEADER_BYTE_1] | [HEADER_BYTE_1, HEADER_BYTE_2, ..] => break,
                [HEADER_BYTE_1, invalid, rest @ ..] => {
                    *input = rest;
                    return Err(SmartAudioError::UnexpectedDataForState(
                        State::AwaitingHeader2,
                        *invalid,
                    ));
                }
                [invalid, rest @ ..] => {
                    *input = rest;
                    return Err(SmartAudioError::UnexpectedDataForState(
                        State::AwaitingHeader1,
                        *invalid,
                    ));
                }
                [] => return Ok(None),
            }
        }

        if input.len() < constants::RESPONSE_PREFIX_SIZE {
            self.fill_buffer_to(input, constants::RESPONSE_PREFIX_SIZE)?;
            return Ok(None);
        }
        let expected_frame_len = match Self::expected_frame_len(input) {
            Ok(length) => length,
            Err(error) => {
                *input = input.split_at(constants::RESPONSE_PREFIX_SIZE).1;
                return Err(error);
            }
        };
        if input.len() < expected_frame_len {
            self.fill_buffer_to(input, expected_frame_len)?;
            return Ok(None);
        }
        let (bytes, rest) = input.split_at(expected_frame_len);
        *input = rest;
        RawSmartAudioFrame::try_from(bytes).map(Some)
    }

    fn expected_frame_len(prefix: &[u8]) -> Result<usize, SmartAudioError> {
        let [_header1, _header2, _command, length, ..] = prefix else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };
        let length = usize::from(*length);
        if !(constants::MIN_RESPONSE_LENGTH..=constants::MAX_RESPONSE_LENGTH).contains(&length) {
            return Err(SmartAudioError::InvalidPayloadLength);
        }
        Ok(constants::RESPONSE_PREFIX_SIZE + length)
    }

    fn fill_buffer_to(
        &mut self,
        input: &mut &[u8],
        target_len: usize,
    ) -> Result<(), SmartAudioError> {
        let count = (target_len - self.buffered_len).min(input.len());
        let (bytes, rest) = input.split_at(count);
        let end = self.buffered_len + count;
        self.buffer
            .get_mut(self.buffered_len..end)
            .ok_or(SmartAudioError::InvalidPayloadLength)?
            .copy_from_slice(bytes);
        self.buffered_len = end;
        *input = rest;
        Ok(())
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

    extern crate std;
    use std::vec::Vec;

    const VALID: &[u8] = &[0xAA, 0x55, 0x02, 0x03, 0, 1, 0x0F];

    fn drain(
        parser: &mut SmartAudioParser,
        mut input: &[u8],
    ) -> Vec<Result<Vec<u8>, SmartAudioError>> {
        let mut results = Vec::new();
        loop {
            let before = input.len();
            match parser.next_frame(&mut input) {
                Ok(Some(frame)) => results.push(Ok(frame.payload().to_vec())),
                Ok(None) => {
                    assert!(input.is_empty());
                    break;
                }
                Err(error) => results.push(Err(error)),
            }
            assert!(
                input.len() < before,
                "every frame or error must advance input"
            );
        }
        results
    }

    #[test]
    fn complete_frames_borrow_input_without_using_buffer() {
        let mut parser = SmartAudioParser::new();
        let mut input = VALID;
        let expected_payload = VALID.get(4..6).unwrap();
        let frame = parser.next_frame(&mut input).unwrap().unwrap();
        assert_eq!(frame.payload().as_ptr(), expected_payload.as_ptr());
        assert_eq!(frame.payload(), expected_payload);
        assert!(input.is_empty());
        assert_eq!(parser.buffer, [0; constants::MAX_FRAME_SIZE]);
        assert_eq!(parser.buffered_len, 0);
    }

    #[test]
    fn overlapping_header_resumes_direct_borrowing() {
        let mut parser = SmartAudioParser::new();
        assert!(parser.next_frame(&mut &[0xAA][..]).unwrap().is_none());
        let mut input = VALID;
        let frame = parser.next_frame(&mut input).unwrap().unwrap();
        assert_eq!(frame.payload().as_ptr(), VALID.get(4..).unwrap().as_ptr());
        assert!(input.is_empty());
        assert_eq!(parser.buffered_len, 0);
    }

    #[test]
    fn spanning_frame_borrows_buffer_then_next_frame_borrows_input() {
        let mut parser = SmartAudioParser::new();
        let (prefix, suffix) = VALID.split_at(5);
        assert!(parser.next_frame(&mut &prefix[..]).unwrap().is_none());
        let buffered_payload_ptr = parser.buffer.get(4..).unwrap().as_ptr();
        let mut chunk = suffix.to_vec();
        chunk.extend_from_slice(VALID);
        let mut input = chunk.as_slice();
        let frame = parser.next_frame(&mut input).unwrap().unwrap();
        assert_eq!(frame.payload().as_ptr(), buffered_payload_ptr);
        assert_eq!(frame.payload(), &[0, 1]);
        assert_eq!(input, VALID);
        let expected_ptr = input.get(4..).unwrap().as_ptr();
        let frame = parser.next_frame(&mut input).unwrap().unwrap();
        assert_eq!(frame.payload().as_ptr(), expected_ptr);
        assert!(input.is_empty());
    }

    #[test]
    fn parses_minimum_and_maximum_frames_across_all_three_chunk_splits() {
        for length in [1, constants::MAX_RESPONSE_LENGTH] {
            let mut bytes = std::vec![0xAA, 0x55, 0xFF, length as u8];
            bytes.extend(core::iter::repeat(0xAA).take(length - 1));
            bytes.push(crate::framing::crc8_dvb_s2(bytes.get(2..).unwrap()));
            for first in 0..=bytes.len() {
                for second in first..=bytes.len() {
                    let mut parser = SmartAudioParser::new();
                    let (a, remaining) = bytes.split_at(first);
                    let (b, c) = remaining.split_at(second - first);
                    let mut results = Vec::new();
                    for chunk in [a, &[], b, &[], c] {
                        results.extend(drain(&mut parser, chunk));
                    }
                    assert_eq!(results, std::vec![Ok(std::vec![0xAA; length - 1])]);
                    assert_eq!(parser.buffered_len, 0);
                }
            }
        }
    }

    #[test]
    fn errors_and_recovery_are_independent_of_chunk_boundaries() {
        let mut bytes = std::vec![0, 0xAA, 0xAA, 0x12];
        for length in [0, 29, 255] {
            bytes.extend_from_slice(&[0xAA, 0x55, 0x02, length]);
        }
        bytes.extend_from_slice(&[0xAA, 0x55, 0x02, 0x03, 0, 1, 0x0E]);
        bytes.extend_from_slice(&[0xAA, 0xAA]);
        bytes.extend_from_slice(VALID);
        bytes.extend_from_slice(VALID);
        let expected = std::vec![
            Err(SmartAudioError::UnexpectedDataForState(
                State::AwaitingHeader1,
                0
            )),
            Err(SmartAudioError::UnexpectedDataForState(
                State::AwaitingHeader2,
                0x12
            )),
            Err(SmartAudioError::InvalidPayloadLength),
            Err(SmartAudioError::InvalidPayloadLength),
            Err(SmartAudioError::InvalidPayloadLength),
            Err(SmartAudioError::InvalidCrc {
                calculated_crc: 0x0F,
                frame_crc: 0x0E
            }),
            Ok(std::vec![0, 1]),
            Ok(std::vec![0, 1]),
        ];
        for first in 0..=bytes.len() {
            for second in first..=bytes.len() {
                let mut parser = SmartAudioParser::new();
                let (a, remaining) = bytes.split_at(first);
                let (b, c) = remaining.split_at(second - first);
                let mut results = Vec::new();
                for chunk in [a, b, c] {
                    results.extend(drain(&mut parser, chunk));
                }
                assert_eq!(results, expected, "splits at {first} and {second}");
            }
        }
        let mut parser = SmartAudioParser::new();
        let mut results = Vec::new();
        for byte in &bytes {
            results.extend(drain(&mut parser, core::slice::from_ref(byte)));
        }
        assert_eq!(results, expected);
    }

    #[test]
    fn reset_discards_any_partial_frame() {
        for split in 1..VALID.len() {
            let mut parser = SmartAudioParser::new();
            let (prefix, _) = VALID.split_at(split);
            assert!(drain(&mut parser, prefix).is_empty());
            parser.reset();
            assert_eq!(drain(&mut parser, VALID), std::vec![Ok(std::vec![0, 1])]);
        }
    }

    #[test]
    fn exhausted_transport_buffer_can_be_reused() {
        let mut parser = SmartAudioParser::new();
        let mut transport = [0xAA, 0x55, 0x02, 0x03];
        assert!(parser.next_frame(&mut &transport[..]).unwrap().is_none());
        transport.copy_from_slice(&[0, 1, 0x0F, 0]);
        let mut input = transport.as_slice();
        assert_eq!(
            parser.next_frame(&mut input).unwrap().unwrap().payload(),
            &[0, 1]
        );
        assert_eq!(input, &[0]);
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
            let p = parser.next_frame(&mut &raw_bytes[..]).unwrap().unwrap();
            assert_eq!(p.len(), raw_bytes.len());
            assert_eq!(p.crc(), *raw_bytes.last().unwrap());
        }
    }
}
