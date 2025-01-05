use core::str;
use libc::*;
use log::{error, info};
use rustls::HandshakeType::{Certificate, PrivateKey};
use std::error::Error;
use std::net::TcpListener as StdTcpListener;
use std::os::unix::io::FromRawFd;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

pub fn create_raw_socket(port: u16) -> Result<i32, Box<dyn Error>> {
    unsafe {
        // Create a socket
        // AF_INET specifies the IPv4 address fam
        // SOCK_STREAM indicates that the socket will use TCP
        // 0 is default for TCP
        let socket_fd = socket(AF_INET, SOCK_STREAM, 0);

        if socket_fd < 0 {
            error!("Failed to create socket");
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
            error!("Failed to set socket options");
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
            error!("Failed to bind socket to address");
            std::process::exit(1);
        }

        // Start listening at address
        if listen(socket_fd, 128) < 0 {
            error!("Failed to listen on socket");
            std::process::exit(1);
        }

        info!("Server started listening on port {}", port);
        return Ok(socket_fd);
    }
}

async fn start_server(port: u16) -> Result<(), Box<dyn Error>> {
    let raw_fd = create_raw_socket(port)?;
    let listener: StdTcpListener = unsafe { StdTcpListener::from_raw_fd(raw_fd) };
    let listener: TcpListener = TcpListener::from_std(listener)?;
    let tls_config = load_tls_config()?;
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    info!("Server is ready to accept connections");
    loop {
        let (stream, address) = listener.accept().await?;
        info!("New connection from {}", address);
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(mut tls_stream) => {
                    info!("TLS hanshake for {} success", address);
                    let mut buffer: Vec<u8> = vec![0; 1024];
                    match tls_stream.read(&mut buffer).await {
                        Ok(_) => {
                            println!("Recieved:{}", str::from_utf8(&buffer[0..]).unwrap());
                            tls_stream
                                .write_all(b"HTTP/2 server response")
                                .await
                                .unwrap();
                        }
                        Err(_) => error!("Failed to read incoming stream"),
                    }
                }
                Err(_) => {
                    error!("TLS hanshake failed");
                }
            }
        });
    }
}

/// Load the TLS certificates and private key
fn load_tls_config() -> Result<ServerConfig, Box<dyn Error>> {
    let certs = Certificate(include_bytes!("../server.crt").to_vec());
    let key = PrivateKey(include_bytes!("../server.key").to_vec());

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    return Ok(config);
}
