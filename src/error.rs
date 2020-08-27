use std::fmt::{self, Debug, Display};
use std::num::{ParseFloatError, ParseIntError};
use std::str::Utf8Error;
use std::{io, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidResponsePrefix,
    InvalidResponseCrcSum,
    InvalidResponseFormat,

    InvalidPayload(Option<Box<dyn std::error::Error>>),

    Io(io::Error),
    ParseFloat(ParseFloatError),
    ParseInt(ParseIntError),
    Utf8(Utf8Error),

    // QPIGS
    InvalidDeviceStatus,

    // QPIRI
    InvalidDeviceBatteryType,
    InvalidDeviceInputVoltageRange,
    InvalidDeviceOutputSourcePriority,
    InvalidDeviceChargeSourcePriority,
    InvalidDeviceMachineType,
    InvalidDeviceTopology,
    InvalidDeviceOutputMode,

    // QMOD
    InvalidDeviceMode,

    // QPIWS
    InvalidWarningStatus,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Forward to the debug formatter.
        Debug::fmt(self, f)
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Self {
        Self::Io(inner)
    }
}

impl From<ParseFloatError> for Error {
    fn from(inner: ParseFloatError) -> Self {
        Self::ParseFloat(inner)
    }
}

impl From<ParseIntError> for Error {
    fn from(inner: ParseIntError) -> Self {
        Self::ParseInt(inner)
    }
}

impl From<Utf8Error> for Error {
    fn from(inner: Utf8Error) -> Self {
        Self::Utf8(inner)
    }
}
