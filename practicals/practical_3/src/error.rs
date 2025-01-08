use std::fmt;
use thiserror::Error;

/// HTTP/2 Error codes as defined in RFC 7540
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum Http2ErrorCode {
    NoError = 0x00,
    ProtocolError = 0x01,
    InternalError = 0x02,
    FlowControlError = 0x03,
    SettingsTimeout = 0x04,
    StreamClosed = 0x05,
    FrameSizeError = 0x06,
    RefusedStream = 0x07,
    Cancel = 0x08,
    CompressionError = 0x09,
    ConnectError = 0x0a,
    EnhanceYourCalm = 0x0b,
    InadequateSecurity = 0x0c,
    Http11Required = 0x0d,
}

impl fmt::Display for Http2ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            Http2ErrorCode::NoError => "No error",
            Http2ErrorCode::ProtocolError => "Protocol error",
            Http2ErrorCode::InternalError => "Internal error",
            Http2ErrorCode::FlowControlError => "Flow control error",
            Http2ErrorCode::SettingsTimeout => "Settings timeout",
            Http2ErrorCode::StreamClosed => "Stream closed",
            Http2ErrorCode::FrameSizeError => "Frame size error",
            Http2ErrorCode::RefusedStream => "Refused stream",
            Http2ErrorCode::Cancel => "Cancel",
            Http2ErrorCode::CompressionError => "Compression error",
            Http2ErrorCode::ConnectError => "Connect error",
            Http2ErrorCode::EnhanceYourCalm => "Enhance your calm",
            Http2ErrorCode::InadequateSecurity => "Inadequate security",
            Http2ErrorCode::Http11Required => "HTTP/1.1 required",
        };
        write!(f, "{}", description)
    }
}

impl Http2ErrorCode {
    /// Convert an integer to an `Http2ErrorCode`.
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0x0 => Some(Http2ErrorCode::NoError),
            0x1 => Some(Http2ErrorCode::ProtocolError),
            0x2 => Some(Http2ErrorCode::InternalError),
            0x3 => Some(Http2ErrorCode::FlowControlError),
            0x4 => Some(Http2ErrorCode::SettingsTimeout),
            0x5 => Some(Http2ErrorCode::StreamClosed),
            0x6 => Some(Http2ErrorCode::FrameSizeError),
            0x7 => Some(Http2ErrorCode::RefusedStream),
            0x8 => Some(Http2ErrorCode::Cancel),
            0x9 => Some(Http2ErrorCode::CompressionError),
            0xa => Some(Http2ErrorCode::ConnectError),
            0xb => Some(Http2ErrorCode::EnhanceYourCalm),
            0xc => Some(Http2ErrorCode::InadequateSecurity),
            0xd => Some(Http2ErrorCode::Http11Required),
            _ => None,
        }
    }

    /// Convert an `Http2ErrorCode` to its integer representation.
    pub fn as_code(self) -> u8 {
        match self {
            Http2ErrorCode::NoError => 0x0,
            Http2ErrorCode::ProtocolError => 0x1,
            Http2ErrorCode::InternalError => 0x2,
            Http2ErrorCode::FlowControlError => 0x3,
            Http2ErrorCode::SettingsTimeout => 0x4,
            Http2ErrorCode::StreamClosed => 0x5,
            Http2ErrorCode::FrameSizeError => 0x6,
            Http2ErrorCode::RefusedStream => 0x7,
            Http2ErrorCode::Cancel => 0x8,
            Http2ErrorCode::CompressionError => 0x9,
            Http2ErrorCode::ConnectError => 0xa,
            Http2ErrorCode::EnhanceYourCalm => 0xb,
            Http2ErrorCode::InadequateSecurity => 0xc,
            Http2ErrorCode::Http11Required => 0xd,
        }
    }
}

#[derive(Error, Debug)]
pub enum Http2Error {
    #[error("HTTP/2 error: {0}")]
    GeneralError(Http2ErrorCode),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid frame: {0}")]
    InvalidFrame(String),
}

/*impl fmt::Display for Http2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Http2Error::GeneralError(s) => write!(f, "General error: {}", s),
            Http2Error::IoError(s) => write!(f, "I/O error: {}", s),
            Http2Error::InvalidFrame(s) => write!(f, "Invalid frame error: {}", s),
        }
    }
}*/

impl Http2Error {
    /// Create a new HTTP/2 general error.
    pub fn new_general_error(code: Http2ErrorCode) -> Self {
        Http2Error::GeneralError(code)
    }

    /// Convert an `Http2ErrorCode` to an `Http2Error`.
    pub fn from_code(code: u8) -> Self {
        match Http2ErrorCode::from_code(code) {
            Some(error_code) => Http2Error::GeneralError(error_code),
            None => Http2Error::InvalidFrame(format!("Unknown error code: {:#x}", code)),
        }
    }
}
