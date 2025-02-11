/// All raw socket creation happens here.
/// this modules is for any thing related to the socket connection
pub mod connection {
    use libc::*;
    use libc::{setsockopt, socket, AF_INET, SOCK_STREAM, SOL_SOCKET, SO_REUSEADDR};
    use log::{error, info};
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    use rustls::ServerConfig;
    use std::error::Error;
    use std::net::TcpListener as StdTcpListener;
    use std::os::unix::io::FromRawFd;
    use std::path::PathBuf;
    use tokio::net::TcpListener;

    fn create_raw_socket(port: u16) -> Result<i32, Box<dyn Error>> {
        unsafe {
            // Create a socket
            // AF_INET specifies the IPv4 address fam
            // SOCK_STREAM indicates that the socket will use TCP
            // 0 is default for TCP
            let socket_fd = socket(AF_INET, SOCK_STREAM, 0);

            if socket_fd < 0 {
                error!(target: "error_logger","Failed to create socket");
                std::process::exit(1);
            }

            // Set socket options
            let option_val: i32 = 1;
            if setsockopt(
                socket_fd,
                SOL_SOCKET,
                SO_REUSEADDR,
                &option_val as *const _ as *const c_void,
                std::mem::size_of_val(&option_val) as u32,
            ) < 0
            {
                error!(target: "error_logger","Failed to set socket options");
                std::process::exit(1);
            }

            // Bind socket to address
            let address = sockaddr_in {
                sin_family: AF_INET as u16,
                sin_port: htons(port),
                sin_addr: in_addr { s_addr: INADDR_ANY },
                sin_zero: [0; 8],
            };

            if bind(
                socket_fd,
                &address as *const sockaddr_in as *const sockaddr,
                std::mem::size_of::<sockaddr_in>() as u32,
            ) < 0
            {
                error!(target: "error_logger","Failed to bind socket to address");
                std::process::exit(1);
            }

            // Start listening at address
            if listen(socket_fd, 128) < 0 {
                error!(target:"error_logger","Failed to listen on socket");
                std::process::exit(1);
            }

            info!(target:"request_logger","Server started listening on port {}", port);
            Ok(socket_fd)
        }
    }

    pub async fn load_tls_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
        let cert_path: PathBuf = PathBuf::from("server.crt");
        let key_path: PathBuf = PathBuf::from("server.key");

        let certs = vec![match CertificateDer::from_pem_file(&cert_path) {
            Ok(c) => c,
            Err(_) => {
                error!(target:"error_logger","Cannot open certificate file");
                std::process::exit(1);
            }
        }];

        let key = match PrivateKeyDer::from_pem_file(&key_path) {
            Ok(k) => k,
            Err(_) => {
                error!(target: "error_logger","Cannot open pk file");
                std::process::exit(1);
            }
        };

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        info!(target: "request_logger","TLS certificate and keys configured");
        Ok(config)
    }

    /// Converts a raw libc socket into a tokio TcpListener
    pub fn get_listener(port: u16) -> Result<TcpListener, Box<dyn std::error::Error>> {
        let raw_fd = create_raw_socket(port)?;
        let listener: StdTcpListener = unsafe { StdTcpListener::from_raw_fd(raw_fd) };
        Ok(TcpListener::from_std(listener)?)
    }
}
