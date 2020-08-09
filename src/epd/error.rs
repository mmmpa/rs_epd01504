use poor_gpio::GpioError;

#[derive(Debug)]
pub enum EpdError {
    SomethingWrong(String),
    DataTooLong(usize, usize),
    GpioError(poor_gpio::GpioError),
    SpiIoError(std::io::Error),
}

impl std::fmt::Display for EpdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EpdError {}

impl From<poor_gpio::GpioError> for EpdError {
    fn from(e: GpioError) -> Self {
        Self::GpioError(e)
    }
}

impl From<std::io::Error> for EpdError {
    fn from(e: std::io::Error) -> Self {
        Self::SpiIoError(e)
    }
}
