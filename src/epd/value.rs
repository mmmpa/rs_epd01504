/// official manual: http://www.e-paper-display.com/downloadRepository/bcd47ebb-5bb9-4fb8-8a59-f90a6d9de473.pdf

pub(crate) const RESET_TIME: u64 = 200;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Command {
    DriverOutputControl = 0x01,
    BoosterSoftStartControl = 0x0C,
    GateScanStartPosition = 0x0F,
    DeepSleepMode = 0x10,
    DataEntryModeSetting = 0x11,
    SwReset = 0x12,
    TemperatureSensorControl = 0x1A,
    MasterActivation = 0x20,
    DisplayUpdateControl1 = 0x21,
    DisplayUpdateControl2 = 0x22,
    WriteRam = 0x24,
    WriteVcomRegister = 0x2C,
    WriteLutRegister = 0x32,
    SetDummyLinePeriod = 0x3A,
    SetGateTime = 0x3B,
    BorderWaveformControl = 0x3C,
    SetRamXAddressStartEndPosition = 0x44,
    SetRamYAddressStartEndPosition = 0x45,
    SetRamXAddressCounter = 0x4E,
    SetRamYAddressCounter = 0x4F,

    // This has no effect, but can be used to terminate Frame write/read commands.
    NOP = 0xFF,
}

pub(crate) mod deep_sleep {
    pub const NORMAL: u8 = 0x00;
    // demo uses
    pub const ENTER_DEEP_SLEEP: u8 = 0x01;
}

pub(crate) mod display_update {
    pub const ENABLE_CLOCK_SIGNAL: u8 = 0x80;
    pub const ENABLE_CLOCK_SIGNAL_ENABLE_CP: u8 = 0xC0; // demo uses 0c11000000

    pub const TO_INITIAL_DISPLAY: u8 = 0x08;
    pub const TO_PATTERN_DISPLAY: u8 = 0x04;
    // demo uses 0c00000100
    pub const TO_BOTH_DISPLAY: u8 = TO_INITIAL_DISPLAY | TO_PATTERN_DISPLAY;

    pub const DISABLE_CP_DISABLE_CLOCK_SIGNAL: u8 = 0x03;
    // demo uses 0c00000011
    pub const DISABLE_CLOCK_SIGNAL: u8 = 0x01;

    pub const DEMO_USES: u8 = 0xC7; // total demo uses
}

pub(crate) mod lut {
    pub const FULL_UPDATE: [u8; 30] = [
        0x66, 0x66, 0x44, 0x66, 0xAA, 0x11, 0x80, 0x08, 0x11, 0x18, 0x81, 0x18, 0x11, 0x88, 0x11,
        0x88, 0x11, 0x88, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x5F, 0xAF, 0xFF, 0xFF, 0x2F, 0x00,
    ];
    pub const PARTIAL_UPDATE: [u8; 30] = [
        0x10, 0x18, 0x18, 0x28, 0x18, 0x18, 0x18, 0x18, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x13, 0x11, 0x22, 0x63, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
}

pub(crate) mod data_entry_mode {
    pub const Y_DEC_X_DEC: u8 = 0x00;
    pub const Y_DEC_X_INC: u8 = 0x01;
    pub const Y_INC_X_DEC: u8 = 0x02;
    pub const Y_INC_X_INC: u8 = 0x03; // [POR]
}

pub(crate) mod data {
    use crate::epd::data_entry_mode;
    use crate::epd::deep_sleep;
    use crate::epd::display_update;

    // P21
    // pub static ref BoosterSoftStartControl: Vec<u8> = vec![0xCF, 0xCE, 0x8D];
    // from sample
    pub const BOOSTER_SOFT_START_CONTROL: [u8; 3] = [0xD7, 0xD6, 0x9D];
    pub const GATE_SCAN_START_POSITION: [u8; 1] = [0x01];
    pub const DEEP_SLEEP_MODE: [u8; 1] = [deep_sleep::NORMAL];
    pub const DATA_ENTRY_MODE_SETTING_YD_XD: [u8; 1] = [data_entry_mode::Y_INC_X_INC];

    // P22
    pub const TEMPERATURE_SENSOR_CONTROL: [u8; 2] = [0b01111111, 0b11110000];

    // Demo doesn't use this command.
    pub const DISPLAY_UPDATE_CONTROL_1: [u8; 0] = [];

    // P23
    pub const DISPLAY_UPDATE_CONTROL_2: [u8; 1] = [display_update::DEMO_USES];
    pub const WRITE_VCOM_REGISTER: [u8; 1] = [0xA8]; // demo uses

    // P24
    pub const WRITE_LUT_REGISTER: [u8; 1] = [0x01];

    // 4 dummy line per gate.
    // Document uses 0x1B, but demo uses 0x1A
    pub const SET_DUMMY_LINE_PERIOD: [u8; 1] = [0x1A];

    // Document uses 0x0B, but demo uses 0x08
    pub const SET_GATE_TIME: [u8; 1] = [0x08];

    // Demo doesn't use this command.
    pub const BORDER_WAVEFORM_CONTROL: [u8; 1] = [0x01];
}
