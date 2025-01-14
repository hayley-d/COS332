//! A Secure, multi-threaded HTTP/1.1 server implementation using TLS for secure communication.
//! This server demonstrates concurrency management, secure password storage and itergration with
//! external services like PostgreSQL and Redis.
use crate::api::question_api::handle_response;
use crate::question::Question;
use crate::request::http_request::Request;
use crate::response::http_response::Response;
use crate::socket::connection::{get_listener, load_tls_config};
use colored::Colorize;
use log::{error, info};
use std::collections::HashMap;
use std::env;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Notify, Semaphore};
use tokio::time::timeout;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use uuid::Uuid;

/// The default port used by the server if none is specified.
const DEFAULT_PORT: u16 = 7878;

pub struct State {
    pub questions: HashMap<Uuid, Question>,
    pub ids: Vec<Uuid>,
}

/// Sets up the server by initializing the connections and configuring logging.
///
/// # Returns
/// A `Result` object with either and Ok(()) or an Err(Box<dyn std::error::Error>)
pub async fn set_up_server() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = match env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PORT.to_string())
        .parse()
    {
        Ok(p) => p,
        Err(_) => DEFAULT_PORT,
    };

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    print_server_info(port);

    let questions: HashMap<Uuid, Question> = Question::parse_file().await;
    let mut ids: Vec<Uuid> = Vec::new();

    for key in questions.keys() {
        ids.push(key.clone());
    }

    let state: Arc<Mutex<State>> = Arc::new(Mutex::new(State { questions, ids }));

    info!(target: "request_logger","Server Started");
    let _ = start_server(port, state.clone()).await;
    Ok(())
}

/// Starts the server, accepting incoming connections and managing their lifecycles.
///
/// # Arguments
/// - `port`: The port the server is running on.
///
/// # Returns
/// A `Result` object with either and Ok(()) or an Err(Box<dyn std::error::Error>)
async fn start_server(
    port: u16,
    questions: Arc<Mutex<State>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener: TcpListener = get_listener(port)?;

    let tls_config = load_tls_config().await?;
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));

    let shutdown: Arc<Notify> = Arc::new(Notify::new());

    let connections: Arc<Semaphore> = Arc::new(Semaphore::new(15));

    let shutdown_signal = shutdown.clone();

    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_err() {
            eprintln!("Failed to listen for shutdown signal");
            std::process::exit(1);
        } else {
            println!("Recieved shutdown request");
            println!("Waiting for tasks to finish");
            shutdown_signal.notify_one();
            println!("Tasks complete, server shutdown started");
            std::process::exit(0);
        }
    });

    tokio::select! {
        _ = run_server(listener,acceptor,connections.clone(),questions.clone())=> {
        }
        _ = shutdown.notified() => {
                info!(target: "request_logger","Server shutdown signal recieved.");
                println!("Server shutdown signal recieved.");
        }
    }
    while connections.clone().available_permits() != 15 {}
    Ok(())
}

/// Accepts connections from incoming clients and completes the TLS hanshake.
///
/// # Arguments
/// - `listener`: A TcpListener
/// - `acceptor`: A TlsAcceptor to handle the Tls hanshake.
/// - `connections`: A Semaphore for limiting the amout of concurrent connections.
async fn run_server(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    connections: Arc<Semaphore>,
    questions: Arc<Mutex<State>>,
) {
    loop {
        let connections = connections.clone();

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

        let questions = Arc::clone(&questions);

        let handle = tokio::spawn(async move {
            if let Ok(tls_stream) = acceptor.accept(stream).await {
                info!(target: "request_logger","TLS handshake successful with {}", address);

                let _ = handle_connection(tls_stream, address.to_string(), questions.clone()).await;
            } else {
                error!(target: "error_logger","TLS handshake failed with {}", address);
            }

            drop(permit);
        });

        handle.await.unwrap();
    }
}

/// Handles incoming connections, including the TLS handshake and request processing.
///
/// # Arguments
/// - `stream`: A mutable TlsStream.
/// - `address`: The address of the client connected to the server.
/// - `state`: A shared, thread-safe state used for managing server data and caching.
///
/// # Returns
/// A `Result` object with either and Ok(()) or an Err(Box<dyn std::error::Error>)
async fn handle_connection(
    mut stream: TlsStream<TcpStream>,
    address: String,
    questions: Arc<Mutex<State>>,
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

        println!("{}", from_utf8(&buffer[..bytes_read]).unwrap());

        let request: Request = match Request::new(&buffer[..bytes_read], address.clone()) {
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

        let mut response: Response = handle_response(request, questions.clone()).await;

        stream.write_all(&response.to_bytes()).await?;
        stream.flush().await?;
    }
}

/// Prints server information on startup.
///
/// # Arguments
/// - `port`: The port the server is currently running on.
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
