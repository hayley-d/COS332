use core::str;
use libc::*;
use log::{error, info};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::error::Error;
use std::net::TcpListener as StdTcpListener;
use std::os::unix::io::FromRawFd;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::sync::{Notify, Semaphore};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::{TlsAcceptor, TlsStream};

use crate::http::validate_preface;

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

pub async fn start_server(port: u16) -> Result<(), Box<dyn Error>> {
    let raw_fd = create_raw_socket(port)?;
    let listener: StdTcpListener = unsafe { StdTcpListener::from_raw_fd(raw_fd) };
    let listener: TcpListener = TcpListener::from_std(listener)?;
    let tls_config = load_tls_config().await?;

    let acceptor = TlsAcceptor::from(Arc::new(tls_config));

    info!("Server is ready to accept connections");

    let shutdown: Arc<Notify> = Arc::new(Notify::new());
    let active_tasks: Arc<Semaphore> = Arc::new(Semaphore::new(10));

    let shutdown_signal = shutdown.clone();
    tokio::spawn(async move {
        if let Err(_) = signal::ctrl_c().await {
            error!("Failed to listen for shutdown signal");
        }

        info!("Server shutdown signal reveived");
        shutdown_signal.notify_one();
    });

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream,address)) => {
                        info!("New connection from {}",address);
                        let acceptor = acceptor.clone();
                        let active_tasks = active_tasks.clone();
                        let shutdown_signal = shutdown.clone();

                        let permit = active_tasks.clone().acquire_owned().await.unwrap();

                        tokio::spawn(async move {
                            if let Ok(mut tls_stream) = acceptor.accept(stream).await {
                                info!("TLS handshake successful with {}",address);

                                if let Err(e) = handle_connection(tls_stream,address.to_string()).await {
                                    error!("Connection error");
                                }
                            }

                            drop(permit);

                            if active_tasks.available_permits() == 0 {
                                shutdown_signal.notify_one();
                            }
                        });
                    },
                    Err(_) => error!("Failed to accept connection"),
                }
            }
            _ = shutdown.notified() => {
                info!("Server shutdown initiated");
                println!("Received Ctrl+C, shutting down...");
                break;
            }
        }
    }

    info!("Waiting for active tasks to complete");
    while active_tasks.available_permits() != 5 {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        std::process::exit(1);
    }
    return Ok(());
}

/// Load the TLS certificates and private key
async fn load_tls_config() -> Result<ServerConfig, Box<dyn Error>> {
    let cert_path: PathBuf = PathBuf::from("server.crt");
    let key_path: PathBuf = PathBuf::from("server.key");

    let certs =
        vec![CertificateDer::from_pem_file(&cert_path).expect("Cannot open certificate file")];
    let key = PrivateKeyDer::from_pem_file(&key_path).expect("Cannot open pk file");

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    return Ok(config);
}

async fn handle_connection(
    stream: TlsStream<TcpStream>,
    address: String,
) -> Result<(), Box<dyn Error>> {
    let stream = validate_preface(stream).await?;
    info!("Valid HTTP/2 preface received from {}", address);
    todo!();
}
