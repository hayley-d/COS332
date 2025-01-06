use std::error::Error;

pub async fn validate_preface(
    mut stream: TlsStream<TcpStream>,
) -> Result<TlsStream<TcpStream>, Box<dyn Error>> {
    const MAGIC: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

    let mut buffer: [u8; 24] = [0; 24];

    stream.read_exact(&mut buffer).await?;
    if buffer != MAGIC {
        error!("Invalid HTTP/2 preface");
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid HTTP/2 preface",
        )));
    }
    return Ok(stream);
}
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
enum FrameType {
    Data = 0x0,
    Headers = 0x1,
    Priority = 0x2,
    RstStream = 0x3,
    Settings = 0x4,
    PushPromise = 0x5,
    Ping = 0x6,
    Goaway = 0x7,
    WindowUpdate = 0x8,
    Continuation = 0x9,
}

impl FrameType {
    /// Takes a u8 value and matches it to one of the predefined FrameType variants.
    fn from_u8(value: u8) -> Result<Self, Box<dyn std::error::Error>> {
        match value {
            0x0 => Ok(FrameType::Data),
            0x1 => Ok(FrameType::Headers),
            0x2 => Ok(FrameType::Priority),
            0x3 => Ok(FrameType::RstStream),
            0x4 => Ok(FrameType::Settings),
            0x5 => Ok(FrameType::PushPromise),
            0x6 => Ok(FrameType::Ping),
            0x7 => Ok(FrameType::Goaway),
            0x8 => Ok(FrameType::WindowUpdate),
            0x9 => Ok(FrameType::Continuation),
            _ => Err(format!("Unknown frame type: {}", value).into()),
        }
    }
}

use std::convert::TryFrom;
use std::io;

use log::error;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

impl TryFrom<u8> for FrameType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(FrameType::Data),
            0x1 => Ok(FrameType::Headers),
            0x2 => Ok(FrameType::Priority),
            0x3 => Ok(FrameType::RstStream),
            0x4 => Ok(FrameType::Settings),
            0x5 => Ok(FrameType::PushPromise),
            0x6 => Ok(FrameType::Ping),
            0x7 => Ok(FrameType::Goaway),
            0x8 => Ok(FrameType::WindowUpdate),
            0x9 => Ok(FrameType::Continuation),
            _ => Err(format!("Unknown frame type: {:#X}", value)),
        }
    }
}

#[derive(Debug)]
struct Frame {
    length: u32,
    frame_type: FrameType,
    flags: u8,
    stream_id: u32,
    payload: Vec<u8>,
}

impl Frame {
    fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        if data.len() < 9 {
            return Err("Incomplete frame header".into());
        }

        let length = u32::from_be_bytes([0, data[0], data[1], data[2]]);
        let frame_type = FrameType::from_u8(data[3])?;
        let flags = data[4];
        //let stream_id = u32::from_be_bytes([data[5], data[6], data[7], data[8]] & 0x7FFFFFFF); // Mask reserved bit
        let payload = data[9..].to_vec();

        Ok(Self {
            length,
            frame_type,
            flags,
            stream_id: 0,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_valid_frame_types() {
        assert_eq!(FrameType::try_from(0x0), Ok(FrameType::Data));
        assert_eq!(FrameType::try_from(0x1), Ok(FrameType::Headers));
        assert_eq!(FrameType::try_from(0x2), Ok(FrameType::Priority));
        assert_eq!(FrameType::try_from(0x3), Ok(FrameType::RstStream));
        assert_eq!(FrameType::try_from(0x4), Ok(FrameType::Settings));
        assert_eq!(FrameType::try_from(0x5), Ok(FrameType::PushPromise));
        assert_eq!(FrameType::try_from(0x6), Ok(FrameType::Ping));
        assert_eq!(FrameType::try_from(0x7), Ok(FrameType::Goaway));
        assert_eq!(FrameType::try_from(0x8), Ok(FrameType::WindowUpdate));
        assert_eq!(FrameType::try_from(0x9), Ok(FrameType::Continuation));
    }

    #[test]
    fn test_invalid_frame_type() {
        let result = FrameType::try_from(0xA); // Unknown frame type
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unknown frame type: 0xA");
    }

    #[test]
    fn test_frame_from_bytes_valid() {
        let data: Vec<u8> = vec![
            0x00, 0x00, 0x05, 0x1, 0x0, 0x00, 0x00, 0x00, 0x1, 0x48, 0x54, 0x54, 0x50, 0x2,
        ];

        let frame = Frame::from_bytes(&data).unwrap();
        assert_eq!(frame.length, 5);
        assert_eq!(frame.frame_type, FrameType::Headers);
        assert_eq!(frame.flags, 0x0);
        assert_eq!(frame.stream_id, 1);
        assert_eq!(frame.payload, vec![0x48, 0x54, 0x54, 0x50, 0x2]);
    }

    #[test]
    fn test_frame_from_bytes_invalid() {
        let data: Vec<u8> = vec![0x00];

        let result = Frame::from_bytes(&data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Incomplete frame header");
    }
}
