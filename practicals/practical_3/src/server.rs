use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use colored::Colorize;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Child;
use tokio::sync::{Mutex, Notify, Semaphore};
use tokio::time::timeout;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

use crate::redis_connection::{
    get_cached_content, read_and_cache_page, set_up_redis, start_redis_server, stop_redis_server,
};
use crate::response::Response;
use crate::socket::connection::{get_listener, load_tls_config};
use crate::{handle_response, Clock, Request};

const DEFAULT_PORT: u16 = 7878;

pub struct SharedState {
    pub redis_connection: redis::Connection,
    pub clock: Clock,
}

impl SharedState {
    pub fn new(redis_connection: redis::Connection, clock: Clock) -> Self {
        return SharedState {
            redis_connection,
            clock,
        };
    }

    pub async fn increment_clock(&mut self) -> i64 {
        self.clock.increment_time()
    }

    pub async fn get_cached_content(&mut self, path: PathBuf) -> Option<Vec<u8>> {
        get_cached_content(&mut self.redis_connection, path).await
    }

    pub async fn read_and_cache_page(&mut self, path: PathBuf) -> Vec<u8> {
        read_and_cache_page(&mut self.redis_connection, path).await
    }
}

pub async fn set_up_server() -> Result<(), Box<dyn std::error::Error>> {
    let redis_child: Child = start_redis_server().await;

    let state: Arc<Mutex<SharedState>> = Arc::new(Mutex::new(SharedState::new(
        match set_up_redis() {
            Ok(c) => c,
            _ => std::process::exit(1),
        },
        Clock::new(),
    )));
    println!("{}{}", ">> ".red().bold(), "Redis working: ".cyan(),);

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let port: u16 = match env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PORT.to_string())
        .parse()
    {
        Ok(p) => p,
        Err(_) => DEFAULT_PORT,
    };

    print_server_info(port);

    info!(target: "request_logger","Server Started");
    let _ = start_server(port, state).await;
    let _ = stop_redis_server(redis_child).await;
    Ok(())
}

async fn start_server(
    port: u16,
    state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn std::error::Error>> {
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
        _ = run_server(listener,acceptor,connections.clone(),is_shutting_down.clone(),state.clone())=> {
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
    state: Arc<Mutex<SharedState>>,
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

        let state = state.clone();
        let handle = tokio::spawn(async move {
            if let Ok(tls_stream) = acceptor.accept(stream).await {
                info!(target: "request_logger","TLS handshake successful with {}", address);

                let _ = handle_connection(tls_stream, address.to_string(), state.clone()).await;
            } else {
                error!(target: "error_logger","TLS handshake failed with {}", address);
            }

            drop(permit);

            return;
        });

        handle.await.unwrap();
    }
}

async fn handle_connection(
    mut stream: TlsStream<TcpStream>,
    address: String,
    state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut buffer = [0; 4096];

        // Read request from the client
        let bytes_read = timeout(Duration::from_millis(100), stream.read(&mut buffer)).await;

        let bytes_read = match bytes_read {
            Ok(b) => b?,
            Err(_) => return Ok(()),
        };

        if bytes_read == 0 {
            println!("Client Disconnected");
            return Ok(());
        }

        let request: Request = match Request::new(
            &buffer[..bytes_read],
            address.clone(),
            state.lock().await.increment_clock().await,
        ) {
            Ok(r) => r,
            Err(_) => {
                println!("Unable to parse in request");
                std::process::exit(1);
            }
        };

        println!("{}", request);

        if request.headers.iter().any(|h| h == "Connection: close") {
            println!("Connection closed");
            let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nBye, World!";
            stream.write_all(response).await?;
            stream.flush().await?;

            return Ok(());
        }

        let mut response: Response = handle_response(request).await;

        stream.write_all(&response.to_bytes()).await?;
        stream.flush().await?;
        return Ok(());
    }
}

fn print_server_info(port: u16) {
    println!("{}", "Server started:".cyan());

    println!(
        "{}{}{}",
        ">> ".red().bold(),
        "address: ".cyan(),
        "127.0.0.1".red().bold()
    );

    println!(
        "{}{}{}",
        ">> ".red().bold(),
        "port: ".cyan(),
        port.to_string().red().bold()
    );

    println!(
        "{}{}{}",
        ">> ".red().bold(),
        "HTTP/1.1: ".cyan(),
        "true".red().bold()
    );

    println!(
        "{}{}{}",
        ">> ".red().bold(),
        "shutdown: ".cyan(),
        "ctrl C".red().bold()
    );

    println!(
        "{}{}\n",
        "Server has launched from http://127.0.0.1:".red().bold(),
        port.to_string().red().bold()
    );
}
