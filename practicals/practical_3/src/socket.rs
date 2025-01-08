/// This module provides utility functions for creating and managing sockets.
/// It utilizes the `socket2` crate for advanced socket operations and integrates with
/// Tokio's asynchronous networking capabilities.
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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::net::TcpStream;
    use tokio::sync::{Notify, Semaphore};
    use tokio::task::{JoinHandle, JoinSet};
    use tokio::time::timeout;
    use tokio_rustls::server::TlsStream;
    use tokio_rustls::TlsAcceptor;

    fn create_raw_socket(port: u16) -> Result<i32, Box<dyn Error>> {
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

    async fn load_tls_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
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
        mut stream: TlsStream<TcpStream>,
        address: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 4096];

        // Read request from the client
        let bytes_read = stream.read(&mut buffer).await?;

        if bytes_read == 0 {
            println!("Client Disconnected");
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received request from {}: {}", address, request);

        // Craft a simple HTTP response
        let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
        stream.write_all(response).await?;
        stream.flush().await?;

        return Ok(());
    }

    /// Converts a raw libc socket into a tokio TcpListener
    fn get_listener(port: u16) -> Result<TcpListener, Box<dyn std::error::Error>> {
        let raw_fd = create_raw_socket(port)?;
        let listener: StdTcpListener = unsafe { StdTcpListener::from_raw_fd(raw_fd) };
        return Ok(TcpListener::from_std(listener)?);
    }

    pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let listener: TcpListener = get_listener(port)?;

        let tls_config = load_tls_config().await?;
        let acceptor = TlsAcceptor::from(Arc::new(tls_config));

        println!("Server is listening on https//:127.0.0.1:{port}");

        let shutdown: Arc<Notify> = Arc::new(Notify::new());
        let is_shutting_down = Arc::new(AtomicBool::new(false));

        let connections: Arc<Semaphore> = Arc::new(Semaphore::new(15));

        let shutdown_signal = shutdown.clone();
        let shutdown_flag = is_shutting_down.clone();

        tokio::spawn(async move {
            if let Err(_) = tokio::signal::ctrl_c().await {
                eprintln!("Failed to listen for shutdown signal");
                std::process::exit(1);
            } else {
                shutdown_flag.store(true, Ordering::SeqCst);
                shutdown_signal.notify_one();
            }
        });

        tokio::select! {
            _ = run_server(listener,acceptor,connections.clone(),is_shutting_down.clone())=> {
            }
            _ = shutdown.notified() => {
                    info!("Server shutdown signal recieved.");
                    println!("Received Ctrl+C, shutting down...");
            }
        }

        info!("Waiting for active tasks to complete");
        while connections.clone().available_permits() != 15 {
            println!("waiting...");
        }

        info!("ALl tasks have completed. Server shutting down");
        println!("ALl tasks have completed. Server shutting down");
        return Ok(());
    }

    async fn run_server(
        listener: TcpListener,
        acceptor: TlsAcceptor,
        connections: Arc<Semaphore>,
        is_shutdown: Arc<AtomicBool>,
    ) {
        loop {
            let connections = connections.clone();
            if is_shutdown.load(Ordering::SeqCst) {
                return;
            }

            let permit = connections.clone().acquire_owned().await.unwrap();
            println!("Permit aquired");

            let (stream, address) = match listener.accept().await {
                Ok((s, a)) => (s, a),
                Err(_) => {
                    error!("Problem establishing connection");
                    continue;
                }
            };

            info!("New connection from {}", address);
            let acceptor = acceptor.clone();

            tokio::spawn(async move {
                if let Ok(tls_stream) = acceptor.accept(stream).await {
                    info!("TLS handshake successful with {}", address);

                    if let Err(_) = handle_connection(tls_stream, address.to_string()).await {
                        error!("Connection error");
                    }

                    println!("Finished task");
                } else {
                    eprintln!("TLS handshake failed with {}", address);
                }

                println!("Dropping permit");
                drop(permit);
            });
        }
    }
}
