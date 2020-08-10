/// official manual: http://www.e-paper-display.com/downloadRepository/bcd47ebb-5bb9-4fb8-8a59-f90a6d9de473.pdf

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Command {
    // value is computed by canvas
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

pub(crate) mod static_data {
    // P21
    // pub static ref BoosterSoftStartControl: Vec<u8> = vec![0xCF, 0xCE, 0x8D];
    // from sample
    pub const BOOSTER_SOFT_START_CONTROL: [u8; 3] = [0xD7, 0xD6, 0x9D];

    // P22
    pub const TEMPERATURE_SENSOR_CONTROL: [u8; 2] = [0b01111111, 0b11110000];

    // Demo doesn't use this command.
    pub const DISPLAY_UPDATE_CONTROL_1: [u8; 0] = [];

    // P23
    pub const VCOM: u8 = 0xA8; // demo uses

    // 4 dummy line per gate.
    // Document uses 0x1B, but demo uses 0x1A
    pub const SET_DUMMY_LINE_PERIOD: u8 = 0x1A;

    // Document uses 0x0B, but demo uses 0x08
    pub const SET_GATE_TIME: u8 = 0x08;
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum DeepSleep {
    Normal = 0x00,
    EnterDeepSleep = 0x01,
}

impl Default for DeepSleep {
    fn default() -> Self {
        Self::Normal
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum DisplayUpdateEnablingStep {
    ClockSignal = 0b1000_0000,
    ClockSignalThenCp = 0b1100_0000,
}

impl Default for DisplayUpdateEnablingStep {
    fn default() -> Self {
        Self::ClockSignalThenCp
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum DisplayUpdateTarget {
    Initial = 0b0000_1000,
    Pattern = 0b0000_0100,
    Both = 0b0000_1100,
}

impl Default for DisplayUpdateTarget {
    fn default() -> Self {
        Self::Pattern
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum DisplayUpdateDisablingStep {
    CpThenClockSignal = 0b0000_0011,
    ClockSignal = 0b0000_0001,
}

impl Default for DisplayUpdateDisablingStep {
    fn default() -> Self {
        Self::CpThenClockSignal
    }
}

const LUT_FULL_UPDATE: [u8; 30] = [
    0x66, 0x66, 0x44, 0x66, 0xAA, 0x11, 0x80, 0x08, 0x11, 0x18, 0x81, 0x18, 0x11, 0x88, 0x11, 0x88,
    0x11, 0x88, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x5F, 0xAF, 0xFF, 0xFF, 0x2F, 0x00,
];

const LUT_PARTIAL_UPDATE: [u8; 30] = [
    0x10, 0x18, 0x18, 0x28, 0x18, 0x18, 0x18, 0x18, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x13, 0x11, 0x22, 0x63, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[derive(Copy, Clone, Debug)]
pub enum Lut {
    FullUpdate,
    PartialUpdate,
}

impl AsRef<[u8]> for Lut {
    fn as_ref(&self) -> &[u8] {
        match self {
            Lut::FullUpdate => &LUT_FULL_UPDATE,
            Lut::PartialUpdate => &LUT_PARTIAL_UPDATE,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum DataEntryAddressDirection {
    YdecXdec = 0b0_00,
    YdecXinc = 0b0_01,
    YincXdec = 0b0_10,
    YincXinc = 0b0_11,
}

impl Default for DataEntryAddressDirection {
    fn default() -> Self {
        Self::YincXinc
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum DataEntryAddressCounterDirection {
    X = 0b0_00,
    Y = 0b1_00,
}

impl Default for DataEntryAddressCounterDirection {
    fn default() -> Self {
        Self::X
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum BorderWaveFormControlFollowSource {
    AtInitialUpdateDisplay = 0b0_0000000,
    AtInitialUpdateDisplayForVbd = 0b1_0000000,
}

impl Default for BorderWaveFormControlFollowSource {
    fn default() -> Self {
        Self::AtInitialUpdateDisplay
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum BorderWaveFormControlSelectGsOrFix {
    GsTransition = 0b0_0_000000,
    FixLevelSetting = 0b0_1_000000,
}

impl Default for BorderWaveFormControlSelectGsOrFix {
    fn default() -> Self {
        Self::FixLevelSetting
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BorderWaveFormControlFixLevelSetting {
    Vss = 0b00_00_0000,
    Vsh = 0b00_01_0000,
    Vsl = 0b00_10_0000,
    Hiz = 0b00_11_0000,
}

impl Default for BorderWaveFormControlFixLevelSetting {
    fn default() -> Self {
        Self::Hiz
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BorderWaveFormControlGsTransitionSetting {
    Gs0Gs1 = 0b000000_01,
    Gs1Gs0 = 0b000000_10,
}

impl Default for BorderWaveFormControlGsTransitionSetting {
    fn default() -> Self {
        Self::Gs0Gs1
    }
}
