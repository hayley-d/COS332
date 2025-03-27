//! A Secure, multi-threaded HTTP/1.1 server implementation using TLS for secure communication.
//! This server demonstrates concurrency management, secure password storage and itergration with
//! external services like PostgreSQL and Redis.
use colored::Colorize;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Notify, Semaphore};

/// The default port used by the server if none is specified.
const DEFAULT_PORT: u16 = 7878;

/// Shared state structure holding connecions to Redis, PostgreSQL and a logical clock.
pub struct SharedState {
    pub(crate) redis_connection: redis::Connection,
    pub(crate) conn: rusqlite::Connection,
    pub(crate) clock: crate::Clock,
    pub(crate) user_states: std::collections::HashMap<uuid::Uuid, UserState>,
}

impl SharedState {
    /// Creates a new `SharedState` instance with the appropriate connections and initializations.
    pub fn new(
        redis_connection: redis::Connection,
        clock: crate::Clock,
        conn: rusqlite::Connection,
    ) -> Self {
        SharedState {
            redis_connection,
            conn,
            clock,
            user_states: std::collections::HashMap::new(),
        }
    }

    // Insert a new user session for session management
    pub fn insert_user(&mut self, session_id: uuid::Uuid) {
        self.user_states.insert(session_id, UserState::new());
    }

    /// Increments the logical clock value.
    pub async fn increment_clock(&mut self) -> i64 {
        self.clock.increment_time()
    }

    /// Retrieves the cached content for a given route name from Redis.
    pub async fn get_cached_content(&mut self, route_name: &str) -> Option<Vec<u8>> {
        crate::redis_connection::get_cached_content(&mut self.redis_connection, route_name).await
    }

    /// Reads and caches a page in Redis based on the path and route name.
    pub async fn read_and_cache_page(&mut self, path: &Path, route_name: &str) -> Vec<u8> {
        crate::redis_connection::read_and_cache_page(&mut self.redis_connection, path, route_name)
            .await
    }

    pub fn add_friend(&mut self, name: &str, number: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO friends (name, number) VALUES (?1, ?2);",
            rusqlite::params![name, number],
        )?;
        Ok(())
    }

    pub fn get_friend(&self, name: &str) -> rusqlite::Result<Option<crate::api::Friend>> {
        let mut stmt = self
            .conn
            .prepare("SELECT number FROM friends WHERE name = ?1;")?;
        let mut rows = stmt.query(rusqlite::params![name])?;
        if let Some(row) = rows.next()? {
            let number: String = row.get(0)?;

            Ok(Some(crate::api::Friend {
                name: name.to_string(),
                number,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn delete_friend(&self, name: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM friends WHERE name = ?1;",
            rusqlite::params![name],
        )?;
        Ok(())
    }

    pub(crate) fn get_all_friends(&self) -> Vec<crate::api::Friend> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, number FROM friends")
            .expect("Failed to prepare");

        let iter = stmt
            .query_map([], |row| {
                Ok(crate::api::Friend {
                    name: row.get(0)?,
                    number: row.get(1)?,
                })
            })
            .expect("Failed to query");

        let mut friends = Vec::new();

        for friend in iter {
            if let Ok(f) = friend {
                friends.push(f);
            }
        }
        friends
    }
}

/// Sets up the server by initializing the connections and configuring logging.
///
/// # Returns
/// A `Result` object with either and Ok(()) or an Err(Box<dyn std::error::Error>)
pub async fn set_up_server() -> Result<(), Box<dyn std::error::Error>> {
    // If no port is provided as command line arg then use 7878 as default
    let port: u16 = match std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PORT.to_string())
        .parse()
    {
        Ok(p) => p,
        Err(_) => DEFAULT_PORT,
    };

    // Open/Create SQLite database
    let conn = match rusqlite::Connection::open("friends.db") {
        Ok(c) => c,
        Err(_) => {
            log::error!(target:"error_logger","Failed to create SQLite connection");
            std::process::exit(1);
        }
    };

    // Create the `friends` table if it doesn't exist already
    conn.execute(
        "CREATE TABLE IF NOT EXISTS friends (
            id TEXT PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            number TEXT NOT NULL
        )",
        [],
    )?;

    let state: Arc<Mutex<SharedState>> = Arc::new(Mutex::new(SharedState::new(
        match crate::redis_connection::set_up_redis() {
            Ok(c) => c,
            _ => {
                log::error!(target:"error_logger","Failed to set up REDIS connection");
                std::process::exit(1);
            }
        },
        crate::Clock::new(),
        conn,
    )));

    println!("{}{}", ">> ".red().bold(), "Redis working: ".cyan(),);

    // For TLS functionality
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Setup logging
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    print_server_info(port);

    log::info!(target: "request_logger","Server Started");
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
    // Get the TCP listener from connection module
    let listener: TcpListener = crate::socket::connection::get_listener(port)?;

    // Seup up TLS for connection
    let tls_config = crate::socket::connection::load_tls_config().await?;
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_config));

    // Set up secure shutdown green thread
    let shutdown: Arc<Notify> = Arc::new(Notify::new());
    let is_shutting_down = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Semaphore limits concurrent connections to at most 15 (green threads)
    let connections: Arc<Semaphore> = Arc::new(Semaphore::new(15));

    let shutdown_signal = shutdown.clone();
    let shutdown_flag = is_shutting_down.clone();

    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_err() {
            eprintln!("Failed to listen for shutdown signal");
            log::error!(target: "error_logger", "Failed to listen for shutdonw signal");
            std::process::exit(1);
        } else {
            println!("Recieved shutdown request");
            println!("Waiting for tasks to finish");
            shutdown_flag.store(true, std::sync::atomic::Ordering::SeqCst);
            shutdown_signal.notify_one();
            println!("Tasks complete, server shutdown started");
            std::process::exit(0);
        }
    });

    tokio::select! {
        _ = run_server(listener,acceptor,connections.clone(),is_shutting_down.clone(),state.clone())=> {
        }
        _ = shutdown.notified() => {
                log::info!(target: "request_logger","Server shutdown signal recieved.");
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
    acceptor: tokio_rustls::TlsAcceptor,
    connections: Arc<Semaphore>,
    is_shutdown: Arc<std::sync::atomic::AtomicBool>,
    state: Arc<Mutex<SharedState>>,
) {
    loop {
        let connections = connections.clone();

        if Arc::clone(&is_shutdown).load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }

        // Aquire permit from semaphore
        let permit = connections.clone().acquire_owned().await.unwrap();

        let result =
            tokio::time::timeout(std::time::Duration::from_millis(100), listener.accept()).await;

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

        // TLS handshake
        let handle = tokio::spawn(async move {
            if let Ok(tls_stream) = acceptor.accept(stream).await {
                log::info!(target: "request_logger","TLS handshake successful with {}", address);
                let _ = handle_connection(tls_stream, address.to_string(), state.clone()).await;
            } else {
                log::error!(target: "error_logger","TLS handshake failed with {}", address);
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
    mut stream: tokio_rustls::server::TlsStream<TcpStream>,
    address: String,
    state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut request_data: Vec<u8> = Vec::new();
    let mut buffer: [u8; 8192] = [0; 8192];

    loop {
        // Read request from the client
        let bytes_read: usize = match tokio::time::timeout(
            std::time::Duration::from_millis(100),
            stream.read(&mut buffer),
        )
        .await
        {
            Ok(Ok(n)) if n > 0 => n,
            _ => break,
        };

        // Append bytes chunk to request
        request_data.extend_from_slice(&buffer[..bytes_read]);

        // Check if we've received the full HTTP headers (detecting "\r\n\r\n")
        if request_data.windows(4).any(|w: &[u8]| w == b"\r\n\r\n") {
            break;
        }
    }

    // Close connection if nothing is recieved
    if request_data.is_empty() {
        return Ok(());
    }

    // Convert request bytes to a UTF-8 string for header parsing
    let request_text: &str = match std::str::from_utf8(&request_data) {
        Ok(text) => text,
        Err(_) => {
            log::error!(target: "error_logger", "Received invalid UTF-8 request");
            return Ok(());
        }
    };

    println!("{}", request_text);

    let request: crate::Request = match crate::Request::new(
        &request_data[..],
        address.clone(),
        state.lock().await.increment_clock().await,
    ) {
        Ok(r) => r,
        Err(_) => {
            log::error!(target:"error_logger","Failed to parse incomming request");
            std::process::exit(1);
        }
    };

    println!("{}", request);

    // Check if the request contains a "Cookie" header (session management)
    if let Some(index) = request.headers.iter().position(|r| r.starts_with("Cookie")) {
        let cookie_value = request
            .headers
            .get(index)
            .unwrap()
            .split("=")
            .last()
            .unwrap();

        let uuid_str = match uuid::Uuid::parse_str(cookie_value) {
            Ok(uuid) => uuid,
            Err(_) => {
                let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nBye, World!";
                stream.write_all(response).await?;
                stream.flush().await?;
                return Ok(());
            }
        };

        // Handle "Connection: close" header
        if request
            .headers
            .iter()
            .any(|h: &String| h == "Connection: close")
        {
            let response: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nBye, World!";
            stream.write_all(response).await?;
            stream.flush().await?;
            return Ok(());
        }

        let mut response: crate::response::Response =
            crate::handle_response(request, state.clone(), uuid_str).await;

        stream.write_all(&response.to_bytes()).await?;
        stream.flush().await?;
    } else {
        let session_id: uuid::Uuid = uuid::Uuid::new_v4();
        let user_state: UserState = UserState::new();

        state
            .lock()
            .await
            .user_states
            .insert(session_id, user_state);

        let mut response: crate::response::Response =
            crate::handle_response(request, state.clone(), session_id).await;

        response.add_header(
            String::from("Set-Cookie"),
            format!("session_id={session_id}"),
        );

        stream.write_all(&response.to_bytes()).await?;
        stream.flush().await?;
    }

    Ok(())
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

pub struct UserState {
    pub(crate) value: f64,
    pub(crate) operator_buffer: std::collections::VecDeque<String>,
}

impl UserState {
    pub fn new() -> UserState {
        UserState {
            value: 0.0,
            operator_buffer: std::collections::VecDeque::new(),
        }
    }

    pub fn buffer(&mut self, operator: String) {
        self.operator_buffer.push_back(operator)
    }

    pub fn pop(&mut self) -> Option<String> {
        self.operator_buffer.pop_front()
    }
}
