//! SmartAudio framing rules for each direction.
//!
//! Host-to-VTX commands encode payload length only and compute CRC over the
//! header, command, length, and payload. The command byte is already encoded
//! as `(command_id << 1) | 1` when passed to [`frame_payload`].
//!
//! VTX-to-host responses encode payload length plus one CRC byte. Their CRC
//! covers command, length, and payload, excluding the two header bytes.
//! [`RawSmartAudioFrame`] validates response framing, not command payload semantics.

use crate::constants;
use crate::SmartAudioError;

pub(crate) fn crc8_dvb_s2(data: &[u8]) -> u8 {
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

/// Encode a host-to-VTX frame. Payloads longer than 255 bytes are rejected
/// with `InvalidPayloadLength` before the buffer is modified.
pub fn frame_payload(
    buffer: &mut [u8],
    command: u8,
    payload: &[u8],
) -> Result<usize, SmartAudioError> {
    let payload_size = payload.len();
    let payload_length =
        u8::try_from(payload_size).map_err(|_error| SmartAudioError::InvalidPayloadLength)?;
    let frame_len = 2 + 1 + 1 + payload_size + 1;
    if buffer.len() < frame_len {
        return Err(SmartAudioError::BufferTooSmall(frame_len));
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
    *len = payload_length;
    body.copy_from_slice(payload);

    let crc = crc8_dvb_s2(head);
    let [crc_byte] = crc_slot else {
        return Err(SmartAudioError::InvalidPayloadLength);
    };
    *crc_byte = crc;
    Ok(frame_len)
}

/// A validated VTX-to-host response frame borrowed from input or parser storage.
///
/// Header, length, and CRC are checked before construction. The payload excludes
/// CRC but includes a reserved byte for setter responses. Use typed conversions
/// to validate the command-specific payload shape. Use [`Self::try_from`] to
/// validate a standalone response slice containing exactly one frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawSmartAudioFrame<'a> {
    bytes: &'a [u8],
    command: u8,
    payload: &'a [u8],
    crc: u8,
}

impl<'a> TryFrom<&'a [u8]> for RawSmartAudioFrame<'a> {
    type Error = SmartAudioError;

    /// Validate exactly one complete VTX-to-host response without copying it.
    /// Checks header, length, and CRC; command-specific payload checks are left
    /// to typed response conversions. Trailing bytes are rejected.
    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        let [h1, h2, command, length, tail @ ..] = bytes else {
            return Err(SmartAudioError::InvalidPayloadLength);
        };
        if (*h1, *h2) != (constants::HEADER_BYTE_1, constants::HEADER_BYTE_2) {
            return Err(SmartAudioError::InvalidHeader);
        }
        let response_length = usize::from(*length);
        if !(constants::MIN_RESPONSE_LENGTH..=constants::MAX_RESPONSE_LENGTH)
            .contains(&response_length)
            || tail.len() != response_length
        {
            return Err(SmartAudioError::InvalidPayloadLength);
        }
        let (&crc, payload) = tail
            .split_last()
            .ok_or(SmartAudioError::InvalidPayloadLength)?;
        let crc_data = bytes
            .get(2..bytes.len() - 1)
            .ok_or(SmartAudioError::InvalidPayloadLength)?;
        let calculated_crc = crc8_dvb_s2(crc_data);
        if crc != calculated_crc {
            return Err(SmartAudioError::InvalidCrc {
                calculated_crc,
                frame_crc: crc,
            });
        }
        Ok(Self {
            bytes,
            command: *command,
            payload,
            crc,
        })
    }
}

impl RawSmartAudioFrame<'_> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_response_framing_before_construction() {
        let valid = [0xAA, 0x55, 0x02, 0x03, 0, 1, 0x0F];
        let frame = RawSmartAudioFrame::try_from(valid.as_slice()).unwrap();
        assert_eq!(frame.command(), 0x02);
        assert_eq!(frame.payload(), &[0, 1]);
        assert_eq!(frame.crc(), 0x0F);

        for end in 0..valid.len() {
            assert_eq!(
                RawSmartAudioFrame::try_from(&valid[..end]),
                Err(SmartAudioError::InvalidPayloadLength)
            );
        }
        let mut invalid = valid;
        invalid[0] = 0;
        assert_eq!(
            RawSmartAudioFrame::try_from(invalid.as_slice()),
            Err(SmartAudioError::InvalidHeader)
        );
        for length in [0, 2, 4, 29] {
            let mut invalid = valid;
            invalid[3] = length;
            assert_eq!(
                RawSmartAudioFrame::try_from(invalid.as_slice()),
                Err(SmartAudioError::InvalidPayloadLength)
            );
        }
        let mut invalid = valid;
        invalid[6] = 0x0E;
        assert_eq!(
            RawSmartAudioFrame::try_from(invalid.as_slice()),
            Err(SmartAudioError::InvalidCrc {
                calculated_crc: 0x0F,
                frame_crc: 0x0E,
            })
        );
        let extra_byte = [0xAA, 0x55, 0x02, 0x03, 0, 1, 0x0F, 0];
        assert_eq!(
            RawSmartAudioFrame::try_from(extra_byte.as_slice()),
            Err(SmartAudioError::InvalidPayloadLength)
        );
    }

    #[test]
    fn reports_required_buffer_size_without_writing() {
        for available in [0, 4] {
            let mut buffer = [0xCC; 4];
            assert_eq!(
                frame_payload(&mut buffer[..available], 0x03, &[]),
                Err(SmartAudioError::BufferTooSmall(5))
            );
            assert_eq!(buffer, [0xCC; 4]);
        }
    }

    #[test]
    fn rejects_payload_lengths_above_u8_without_writing() {
        let payload = [0x42; 512];
        for payload_len in [256, 257, 512] {
            let mut buffer = [0xCC; 517];
            assert_eq!(
                frame_payload(&mut buffer, 0x05, &payload[..payload_len]),
                Err(SmartAudioError::InvalidPayloadLength)
            );
            assert_eq!(buffer, [0xCC; 517]);
        }

        let mut small_buffer = [0xCC; 4];
        assert_eq!(
            frame_payload(&mut small_buffer, 0x05, &payload),
            Err(SmartAudioError::InvalidPayloadLength)
        );
        assert_eq!(small_buffer, [0xCC; 4]);
    }

    #[test]
    fn encodes_maximum_u8_payload_length() {
        let payload = [0x42; 255];
        let mut buffer = [0xCC; 261];
        assert_eq!(frame_payload(&mut buffer, 0x05, &payload), Ok(260));
        assert_eq!(&buffer[..4], &[0xAA, 0x55, 0x05, 0xFF]);
        assert_eq!(&buffer[4..259], &payload);
        assert_eq!(buffer[259], 0xDE);
        assert_eq!(buffer[260], 0xCC);
    }
}
