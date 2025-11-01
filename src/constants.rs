// Constants for SmartAudio protocol
pub(crate) const HEADER_BYTE_1: u8 = 0xAA;
pub(crate) const HEADER_BYTE_2: u8 = 0x55;
pub(crate) const MAX_FRAME_SIZE: usize = 32;
pub(crate) const MAX_PAYLOAD_SIZE: usize = 28;
pub(crate) const MIN_PAYLOAD_SIZE: usize = 3;

// Bitmasks for GetSettings response payload
pub(crate) mod get_settings_flags {
    pub const USER_FREQUENCY: u8 = 0x01;
    pub const PITMODE_ENABLED: u8 = 0x02;
    pub const PITMODE_IN_RANGE: u8 = 0x04;
    pub const PITMODE_OUT_RANGE: u8 = 0x08;
    pub const UNLOCKED: u8 = 0x10;
}

// Bitmasks for SetMode command payload
pub(crate) mod mode_flags {
    pub const PITMODE_IN_RANGE: u8 = 0x01;
    pub const PITMODE_OUT_RANGE: u8 = 0x02;
    pub const PITMODE_ENABLED: u8 = 0x04;
    pub const UNLOCKED: u8 = 0x08;
}

// Command bytes sent to the VTX
pub(crate) mod command {
    pub const GET_SETTINGS: u8 = 0x03;
    pub const SET_POWER: u8 = 0x05;
    pub const SET_CHANNEL: u8 = 0x07;
    pub const SET_FREQUENCY: u8 = 0x09;
    pub const SET_MODE: u8 = 0x0B;
}

// Responses bytes sent from VTX
pub(crate) mod response {
    pub const GET_SETTINGS_V1_0: u8 = 0x01;
    pub const GET_SETTINGS_V2_0: u8 = 0x09;
    pub const GET_SETTINGS_V2_1: u8 = 0x11;
    pub const SET_POWER: u8 = 0x02;
    pub const SET_CHANNEL: u8 = 0x03;
    pub const SET_FREQUENCY: u8 = 0x04;
    pub const SET_MODE: u8 = 0x05;
}
