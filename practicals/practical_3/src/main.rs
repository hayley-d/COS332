use colored::Colorize;
use log::error;
use practical_3::socket::connection::start_server;
use practical_3::{ErrorType, Logger};
use std::env;

const DEFAULT_PORT: u16 = 7878;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    print_server_info(port);

    let _ = start_server(port).await;

    Ok(())
}

/*async fn run_server(mut listener: Listener, clock: Arc<Mutex<Clock>>) -> Result<(), ErrorType> {
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
}*/

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
