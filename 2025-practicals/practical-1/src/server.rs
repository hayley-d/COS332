//! A Secure, multi-threaded HTTP/1.1 server implementation using TLS for secure communication.
//! This server demonstrates concurrency management, secure password storage and itergration with
//! external services like PostgreSQL and Redis.
use crate::redis_connection::{get_cached_content, read_and_cache_page, set_up_redis};
use crate::response::Response;
use crate::socket::connection::{get_listener, load_tls_config};
use crate::{handle_response, Clock, ErrorType, Request};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use colored::Colorize;
use dotenv::dotenv;
use log::{error, info};
use rand::rngs::OsRng;
use std::env;
use std::path::Path;
use std::str::from_utf8;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Notify, Semaphore};
use tokio::time::timeout;
use tokio_postgres::{Client, NoTls};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use uuid::Uuid;

/// The default port used by the server if none is specified.
const DEFAULT_PORT: u16 = 7878;

/// Shared state structure holding connecions to Redis, PostgreSQL and a logical clock.
pub struct SharedState {
    pub redis_connection: redis::Connection,
    pub clock: Clock,
    pub client: Client,
}

impl SharedState {
    /// Creates a new `SharedState` instance.
    pub fn new(redis_connection: redis::Connection, clock: Clock, client: Client) -> Self {
        SharedState {
            redis_connection,
            clock,
            client,
        }
    }

    /// Increments the logical clock value.
    pub async fn increment_clock(&mut self) -> i64 {
        self.clock.increment_time()
    }

    /// Retrieves the cached content for a given route name from Redis.
    pub async fn get_cached_content(&mut self, route_name: &str) -> Option<Vec<u8>> {
        get_cached_content(&mut self.redis_connection, route_name).await
    }

    /// Reads and caches a page in Redis based on the path and route name.
    pub async fn read_and_cache_page(&mut self, path: &Path, route_name: &str) -> Vec<u8> {
        read_and_cache_page(&mut self.redis_connection, path, route_name).await
    }

    /// Adds a new user to the PostgreSQL database, hashing their password.
    ///
    /// # Arguments
    /// - `username`: The user's username
    /// - `password`: The user's plain text password.
    ///
    /// # Returns
    /// A `Response` object with either Ok(session_id) or an Err(Box<dyn std::error::Error>)
    pub async fn add_user(
        &mut self,
        username: String,
        password: String,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let hash = Self::hash_password(&password).unwrap();
        let session_id = Uuid::new_v4();

        let query = self
            .client
            .prepare("INSERT INTO users (username, password, session_id) VALUES ($1,$2,$3);")
            .await?;

        let _ = self
            .client
            .execute(&query, &[&username, &hash, &session_id])
            .await?;

        Ok(session_id)
    }

    /// Finds an existing user by username and returns their session ID.
    pub async fn find_user(
        &mut self,
        username: String,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let query = self
            .client
            .prepare("SELECT * FROM users WHERE username = $1")
            .await?;

        let row = self.client.query_one(&query, &[&username]).await?;

        if row.is_empty() {
            return Err(Box::new(ErrorType::ReadError(
                "Failed to find user".to_string(),
            )));
        }

        let session_id: Uuid = row.get(4);

        Ok(session_id)
    }

    /// Hashes a password using the Argon2 algorithm and a random salt.
    fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        match argon2.hash_password(password.as_bytes(), salt.as_salt()) {
            Ok(hash) => Ok(hash.to_string()),
            Err(_) => {
                error!(target: "error_logger","Failed to create new user");
                std::process::exit(1);
            }
        }
    }
}

/// Sets up the server by initializing the connections and configuring logging.
///
/// # Returns
/// A `Result` object with either and Ok(()) or an Err(Box<dyn std::error::Error>)
pub async fn set_up_server() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}{}", ">> ".red().bold(), "Redis working: ".cyan(),);

    let port: u16 = match env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PORT.to_string())
        .parse()
    {
        Ok(p) => p,
        Err(_) => DEFAULT_PORT,
    };

    match dotenv() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("dotenv error: {:?}", e);
            std::process::exit(1);
        }
    };

    let database_url = match env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(e) => {
            eprintln!("DATABASE_URL must be set in .env file: {:?}", e);
            std::process::exit(1);
        }
    };

    let (client, connection) = match tokio_postgres::connect(&database_url, NoTls).await {
        Ok((c, con)) => (c, con),
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
            std::process::exit(1);
        }
    });

    let state: Arc<Mutex<SharedState>> = Arc::new(Mutex::new(SharedState::new(
        match set_up_redis() {
            Ok(c) => c,
            _ => std::process::exit(1),
        },
        Clock::new(),
        client,
    )));

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    print_server_info(port);

    info!(target: "request_logger","Server Started");
    let _ = start_server(port, state).await;
    Ok(())
}

/// Starts the server, accepting incoming connections and managing their lifecycles.
///
/// # Arguments
/// - `port`: The port the server is running on.
/// - `state`: A shared, thread-safe state used for managing server data and caching.
///
/// # Returns
/// A `Result` object with either and Ok(()) or an Err(Box<dyn std::error::Error>)
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
        if tokio::signal::ctrl_c().await.is_err() {
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
    Ok(())
}

/// Accepts connections from incoming clients and completes the TLS hanshake.
///
/// # Arguments
/// - `listener`: A TcpListener
/// - `acceptor`: A TlsAcceptor to handle the Tls hanshake.
/// - `connections`: A Semaphore for limiting the amout of concurrent connections.
/// - `is_shutdown`: A thread-safe `AtomicBool` to indicate if the server is in shutdown mode.
/// - `state`: A shared, thread-safe state used for managing server data and caching.
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

        println!("{}", from_utf8(&buffer[..bytes_read]).unwrap());

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

        let mut response: Response = handle_response(request, state.clone()).await;
        println!("Received response from api");

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
