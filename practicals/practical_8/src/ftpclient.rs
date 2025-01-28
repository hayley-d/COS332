use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;
use tokio::net::TcpStream;
#[derive(Debug)]
pub enum MonitorError {
    FtpError(FtpErrorKind),
    FileError(std::io::Error),
    WatchError(notify::Error),
}
impl fmt::Display for MonitorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonitorError::FtpError(s) => write!(f, "FTP Error: {}", s),
            MonitorError::FileError(s) => write!(f, "File Error: {}", s),
            MonitorError::WatchError(s) => write!(f, "Watch Error: {}", s),
        }
    }
}
impl std::error::Error for MonitorError {}

#[derive(Debug)]
pub enum FtpErrorKind {
    ConnectionError(String),
    AuthenticationError(String),
    TransferError(String),
    ProtocolError(String),
    TimeoutError(String),
}

impl fmt::Display for FtpErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FtpErrorKind::ConnectionError(s) => {
                write!(f, "Connection error: {}", s)
            }
            FtpErrorKind::AuthenticationError(s) => {
                write!(f, "Authentication error: {}", s)
            }
            FtpErrorKind::TransferError(s) => {
                write!(f, "Transfer error: {}", s)
            }
            FtpErrorKind::ProtocolError(s) => {
                write!(f, "Protocol error: {}", s)
            }
            FtpErrorKind::TimeoutError(s) => {
                write!(f, "Timeout error: {}", s)
            }
        }
    }
}

// Enhanced FTP client with advanced features
pub struct FtpClient {
    pub(crate) control_stream: TcpStream,
    pub(crate) data_port: u16,
    pub(crate) features: HashMap<String, Vec<String>>, // Store FEAT command responses
    pub(crate) transfer_mode: TransferMode,
    pub(crate) compression: bool,
    pub(crate) tls_enabled: bool,
}

pub enum TransferMode {
    Passive,
    ExtendedPassive,
    Active,
}

impl FtpClient {
    pub async fn new(host: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let mut client = FtpClient {
            control_stream: Self::connect(host, port)?,
            data_port: 0,
            features: HashMap::new(),
            transfer_mode: TransferMode::Passive,
            compression: false,
            tls_enabled: false,
        };

        // Query server features
        client.query_features().await?;

        // Try to enable TLS if available
        if client.supports_feature("AUTH TLS") {
            client.enable_tls().await?;
        }

        // Try to enable MODE Z (compression) if available
        if client.supports_feature("MODE Z") {
            client.enable_compression().await?;
        }

        Ok(client)
    }

    pub async fn query_features(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command("FEAT").await?;
        let response = self.read_multiline_response()?;

        for feature in response.lines() {
            let feat = feature.trim();
            if !feat.starts_with('2') {
                // Skip response code
                self.features.insert(
                    feat.to_string(),
                    vec![], // Could parse feature parameters here
                );
            }
        }
        Ok(())
    }

    async fn send_command(&self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn read_response(&self) -> Result<String, Box<dyn std::error::Error>> {
        todo!()
    }

    /// Enables TLS
    pub async fn enable_tls(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command("AUTH TLS").await?;
        let response = self.read_response().await?;

        if response.starts_with('2') {
            self.tls_enabled = true;
            // Implement TLS things here
        }
        Ok(())
    }

    /// Enables compression
    pub async fn enable_compression(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command("MODE Z").await?;
        let response = self.read_response().await?;

        if response.starts_with('2') {
            self.compression = true;
        }
        Ok(())
    }

    // Support for partial file transfers using REST command
    pub async fn download_partial(
        &mut self,
        remote_path: &str,
        local_path: &str,
        start: u64,
        end: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(&format!("REST {}", start)).await?;
        let response = self.read_response().await?;

        if !response.starts_with('3') {
            return Err(Box::new(MonitorError::FtpError(
                FtpErrorKind::ProtocolError("REST command not supported".into()),
            )));
        }

        // Similar to regular download but with range
        self.download_file(remote_path, local_path)
    }

    // Implement MLSD command for detailed directory listings
    pub async fn list_directory(
        &mut self,
        path: &str,
    ) -> Result<Vec<FileMetadata>, Box<dyn std::error::Error>> {
        self.send_command(&format!("MLSD {}", path)).await?;
        let response = self.read_multiline_response()?;

        let mut metadata = Vec::new();
        for line in response.lines() {
            if let Some(parsed) = Self::parse_mlsd_line(line) {
                metadata.push(parsed);
            }
        }

        Ok(metadata)
    }

    // Support for MFMT command to preserve file timestamps
    pub async fn set_timestamp(
        &mut self,
        path: &str,
        timestamp: SystemTime,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp_str = timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.send_command(&format!("MFMT {} {}", timestamp_str, path))
            .await?;
        self.read_response().await?;
        Ok(())
    }
}
