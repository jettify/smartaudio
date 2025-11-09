use crate::constants::command;
use crate::constants::mode_flags;
use crate::parser::frame_payload;
use crate::parser::SmartAudioError;

pub trait SmartAudioCommand {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, SmartAudioError>;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetSettingsCommand {}

impl SmartAudioCommand for GetSettingsCommand {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, SmartAudioError> {
        const PAYLOAD: [u8; 0] = [];
        frame_payload(buffer, command::GET_SETTINGS, &PAYLOAD)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Power {
    Level(u8),
    dBm(u8),
}

impl Default for Power {
    fn default() -> Self {
        Self::Level(0)
    }
}

impl From<Power> for u8 {
    fn from(power: Power) -> Self {
        match power {
            Power::dBm(value) => value | 0b1000_0000,
            Power::Level(value) => value,
        }
    }
}

pub struct SetPowerCommand {
    pub power: Power,
}

impl SmartAudioCommand for SetPowerCommand {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, SmartAudioError> {
        let payload = [self.power.into()];
        frame_payload(buffer, command::SET_POWER, &payload)
    }
}

pub struct SetChannelCommand {
    pub channel: u8,
}

impl SmartAudioCommand for SetChannelCommand {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, SmartAudioError> {
        let payload = [self.channel];
        frame_payload(buffer, command::SET_CHANNEL, &payload)
    }
}

pub struct SetFrequencyCommand {
    pub frequency: u16,
}

impl SmartAudioCommand for SetFrequencyCommand {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, SmartAudioError> {
        let payload = self.frequency.to_be_bytes();
        frame_payload(buffer, command::SET_FREQUENCY, &payload)
    }
}

pub struct SetModeCommand {
    pub pitmode_in_range_active: bool,
    pub pitmode_out_range_active: bool,
    pub pitmode_enabled: bool,
    pub unlocked: bool,
}

impl SmartAudioCommand for SetModeCommand {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, SmartAudioError> {
        let mode = (u8::from(self.pitmode_in_range_active) * mode_flags::PITMODE_IN_RANGE)
            | (u8::from(self.pitmode_out_range_active) * mode_flags::PITMODE_OUT_RANGE)
            | (u8::from(self.pitmode_enabled) * mode_flags::PITMODE_ENABLED)
            | (u8::from(self.unlocked) * mode_flags::UNLOCKED);
        let payload = [mode];
        frame_payload(buffer, command::SET_MODE, &payload)
    }
}

#[cfg(test)]
mod tesst {
    use super::*;

    #[test]
    fn test_set_channel_command() {
        let frame = SetChannelCommand { channel: 0 };
        let mut buffer: [u8; 6] = [0; 6];
        let size = frame.to_bytes(&mut buffer).unwrap();
        let expected: [u8; 6] = [0xAA, 0x55, 0x07, 0x01, 0x00, 0xB8];
        assert_eq!(expected, buffer[0..size]);
    }

    #[test]
    fn test_set_power_command() {
        // Master SmartAudioV2: Master: 0xAA 0x55 0x05(Command 2) 0x01(Length) 0x00(Power Level) 0x6B(CRC8)
        let frame_2_0 = SetPowerCommand {
            power: Power::Level(0),
        };
        let mut buffer: [u8; 6] = [0; 6];
        let size = frame_2_0.to_bytes(&mut buffer).unwrap();
        let expected: [u8; 6] = [0xAA, 0x55, 0x05, 0x01, 0x00, 0x6B];
        assert_eq!(expected, buffer[0..size]);

        // Master SmartAudioV2.1: Master: 0xAA 0x55 0x05(Command 2) 0x01(Length) 0x8E(14dbm plus MSB set) 0x2C(CRC8)
        let frame_2_1 = SetPowerCommand {
            power: Power::dBm(14),
        };
        let size = frame_2_1.to_bytes(&mut buffer).unwrap();
        let expected: [u8; 6] = [0xAA, 0x55, 0x05, 0x01, 0x8E, 0x2C];
        assert_eq!(expected, buffer[0..size]);
    }
    #[test]
    fn test_get_settings_command() {
        // Master: 0xAA 0x55 0x03(modified Command see Host to VTX ) 0x00, Length 0x9F (CRC)
        let frame = GetSettingsCommand {};
        let mut buffer: [u8; 5] = [0; 5];
        let size = frame.to_bytes(&mut buffer).unwrap();
        let expected: [u8; 5] = [0xAA, 0x55, 0x03, 0x00, 0x9F];
        assert_eq!(expected, buffer[0..size]);
    }

    #[test]
    fn test_set_frequency_command() {
        // Master: 0xAA 0x55 0x09(Command 4) 0x02(Length) 0x16 0xE9(Frequency 5865) 0xDC(CRC8)
        let frame = SetFrequencyCommand { frequency: 5865 };
        let mut buffer: [u8; 7] = [0; 7];
        let size = frame.to_bytes(&mut buffer).unwrap();
        let expected: [u8; 7] = [0xAA, 0x55, 0x09, 0x02, 0x16, 0xE9, 0xDC];
        assert_eq!(expected, buffer[0..size]);
    }

    #[test]
    fn test_set_mode_command() {
        // Master: 0xAA 0x55 0x0B(Command 5) 0x01(Length) 0x0A(Mode) 0x7B(CRC8)
        let frame = SetModeCommand {
            pitmode_in_range_active: false,
            pitmode_out_range_active: true,
            pitmode_enabled: false,
            unlocked: true,
        };
        let mut buffer: [u8; 6] = [0; 6];
        let size = frame.to_bytes(&mut buffer).unwrap();
        let expected: [u8; 6] = [0xAA, 0x55, 0x0B, 0x01, 0x0A, 0x7B];
        assert_eq!(expected, buffer[0..size]);
    }
}
