use colored::Colorize;
use log::{error, info};
use rust_server::connection::connections::*;
use rust_server::error::my_errors::*;
use rust_server::request_validation::handle_request;
use rust_server::{handle_response, my_socket::*, request::*, shutdown::*};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex, Semaphore};
use tokio::time::timeout;

const DEFAULT_PORT: u16 = 7878;

#[tokio::main]
async fn main() -> Result<(), ErrorType> {
    let logger: Logger = Logger::new("server.log");

    let port: u16 = match env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PORT.to_string())
        .parse()
    {
        Ok(p) => p,
        Err(_) => {
            error!("Failed to parse port number");
            let error = ErrorType::SocketError(String::from("Problem parsing port"));
            logger.log_error(&error);
            DEFAULT_PORT
        }
    };

    let socket = match create_socket(port) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create socket");
            logger.log_error(&e);
            panic!(
                "{}",
                "Error creating socket, refer to the server log"
                    .red()
                    .bold()
            );
        }
    };

    // create a listener from the socket
    let listener = match get_listener(socket) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create TCP listener");
            logger.log_error(&e);
            panic!(
                "{}",
                "Error creating listener, refer to the server log"
                    .red()
                    .bold()
            );
        }
    };

    // create a channel
    let (tx, _rx) = broadcast::channel(10);
    let tx = Arc::new(Mutex::new(tx));
    let mut shutdown = Shutdown::new(Arc::clone(&tx));

    // Graceful shutdown using signal handling
    let shutdown_signal = tokio::signal::ctrl_c();

    let listener: Listener = Listener {
        listener,
        connection_limit: Arc::new(Semaphore::new(5)),
        shutdown_tx: Arc::clone(&tx),
    };

    print_server_info(port);

    tokio::select! {
        _ = run_server(listener,Arc::new(Mutex::new(Clock::new()))) => {
            println!("{}","Gracefull shutdown completed successfully.".cyan());
        }
        _ = shutdown_signal => {
            info!("Server shutdown initiated");
            println!("{}{}","WARNING:".yellow().bold()," SIGINT received: Requesting shutdown..".yellow());
            println!("{}","Shutdown requested.\nWaiting for pending I/O...".cyan());
            info!("Server shutdown successful");
            shutdown.initiate_shutdown().await;
        }
    }

    Ok(())
}

async fn run_server(mut listener: Listener, clock: Arc<Mutex<Clock>>) -> Result<(), ErrorType> {
    loop {
        let c = Arc::clone(&clock);

        // Returns an error when the semaphore has been closed, since I do not close it
        // unwrap should be safe
        let permit = listener
            .connection_limit
            .clone()
            .acquire_owned()
            .await
            .unwrap();

        let (client, addr): (TcpStream, SocketAddr) = match listener.accept().await {
            Ok((c, a)) => (c, a.into()),
            Err(_) => {
                error!("Failed to connect to the client");
                return Err(ErrorType::SocketError(String::from(
                    "Error connecting to client",
                )));
            }
        };

        let mut handler = ConnectionHandler {
            stream: client,
            addr,
            shutdown_rx: listener.shutdown_tx.lock().await.subscribe(),
        };

        tokio::spawn(async move {
            loop {
                let lam = Arc::clone(&c);

                let mut buffer: [u8; 4096] = [0; 4096];
                let bytes_read =
                    match timeout(Duration::from_secs(5), handler.stream.read(&mut buffer)).await {
                        Ok(Ok(number_bytes)) if number_bytes == 0 => break,
                        Ok(Ok(number_bytes)) => number_bytes,
                        Ok(Err(_)) => {
                            error!("Failed to connect to client");
                            break;
                        }
                        Err(_) => break,
                    };

                // check request for any potential maliciousness
                match handle_request(&buffer[..bytes_read]) {
                    Ok(_) => (),
                    Err(_) => {
                        error!("Request failed test for possible malicious threats");
                    }
                };

                let request_id: i64 = lam.lock().await.increment_time();

                let request: Request =
                    match Request::new(&buffer[..bytes_read], addr.ip(), request_id) {
                        Ok(r) => {
                            r.print();
                            r
                        }
                        Err(_) => {
                            error!("Failed to create request");
                            break;
                        }
                    };

                let mut response = handle_response(request).await;

                if let Err(_) = handler.stream.write_all(&response.to_bytes()).await {
                    error!("Failed to connect to client");
                }

                if !handler.shutdown_rx.is_empty() {
                    let msg: Message = match handler.shutdown_rx.recv().await {
                        Ok(m) => m,
                        Err(_) => {
                            error!("Failed to receive message from shutdown sender");
                            Message::ServerRunning
                        }
                    };

                    if msg == Message::Terminate {
                        break;
                    }
                }
            }
            drop(permit);
        });
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
