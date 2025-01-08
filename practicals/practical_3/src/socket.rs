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
            return Ok(socket_fd);
        }
    }

    async fn load_tls_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
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
        return Ok(config);
    }

    async fn handle_connection(
        mut stream: TlsStream<TcpStream>,
        address: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut buffer = [0; 4096];

            // Read request from the client
            let bytes_read = stream.read(&mut buffer).await?;

            if bytes_read == 0 {
                println!("Client Disconnected");
                return Ok(());
            }

            let request = String::from_utf8_lossy(&buffer[..bytes_read]);

            println!(
                "Request: method:{}, uri:{}, client IP:{}",
                "GET", "/", address
            );

            if request.contains("Connection: close") {
                return Ok(());
            }

            // Craft a simple HTTP response
            let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
            stream.write_all(response).await?;
            stream.flush().await?;
        }
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
                println!("Recieved shutdown request");
                println!("Waiting for tasks to finish");
                shutdown_flag.store(true, Ordering::SeqCst);
                shutdown_signal.notify_one();
                println!("Tasks complete, server shutdown started");
                std::process::exit(0);
            }
        });

        tokio::select! {
            _ = run_server(listener,acceptor,connections.clone(),is_shutting_down.clone())=> {
            }
            _ = shutdown.notified() => {
                    info!(target: "request_logger","Server shutdown signal recieved.");
                    println!("Server shutdown signal recieved.");
            }
        }
        while connections.clone().available_permits() != 15 {}
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

            if Arc::clone(&is_shutdown).load(Ordering::SeqCst) {
                return;
            }

            let permit = connections.clone().acquire_owned().await.unwrap();

            let result = timeout(Duration::from_millis(100), listener.accept()).await;

            let (stream, address) = match result {
                Ok(Ok((s, a))) => (s, a),
                Ok(Err(_)) => {
                    drop(permit);
                    continue;
                }
                Err(_) => {
                    drop(permit);
                    continue;
                }
            };

            let acceptor = acceptor.clone();

            let handle = tokio::spawn(async move {
                if let Ok(tls_stream) = acceptor.accept(stream).await {
                    info!(target: "request_logger","TLS handshake successful with {}", address);

                    let _ = handle_connection(tls_stream, address.to_string()).await;
                } else {
                    error!(target: "error_logger","TLS handshake failed with {}", address);
                }

                drop(permit);

                return;
            });

            handle.await.unwrap();
        }
    }
}
