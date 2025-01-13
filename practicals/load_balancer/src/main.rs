use bytes::Bytes;
use dotenv::*;
use http_body_util::Full;
use hyper::Response;
use load_balancer::load_balancer::load_balancer::LoadBalancer;
use load_balancer::request::buffer_to_request;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Notify};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut nodes: Vec<String> = get_nodes();

    // Listen on port 3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener: TcpListener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(_) => {
            eprintln!("Failed to bind to socket");
            std::process::exit(1);
        }
    };

    println!("Listening on http://{}", addr);

    let state: Arc<Mutex<LoadBalancer>> = Arc::new(Mutex::new(LoadBalancer::new(&mut nodes).await));

    let graceful = Arc::new(Notify::new());
    let notify_clone = graceful.clone();
    let mut signal = std::pin::pin!(shutdown_signal());
    tokio::spawn(async move {
        shutdown_signal().await;
        notify_clone.notify_waiters();
    });

    tokio::select! {
        _ = reverse_proxy(listener,state.clone()) => {
            println!("loop ended");
        },
        _ = &mut signal => {
            eprintln!("graceful shutdown signal recieved");
        }
        _ = graceful.notified() => {
                eprintln!("Graceful shutdown initiated");
                std::process::exit(0);
            }
    }

    Ok(())
}

async fn reverse_proxy(listener: TcpListener, state: Arc<Mutex<LoadBalancer>>) {
    loop {
        let state = state.clone();
        if let Ok((mut stream, client_address)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buffer: [u8; 4096] = [0; 4096];

                while let Ok(bytes_read) = stream.read(&mut buffer).await {
                    if bytes_read == 0 {
                        return;
                    }

                    let mut request: http::Request<Vec<u8>> =
                        match buffer_to_request(buffer[..bytes_read].to_vec()) {
                            Ok(request) => request,
                            Err(e) => {
                                eprintln!("Failed to parse request: {}", e);
                                send_error_response(400, &mut stream).await;
                                return;
                            }
                        };

                    // Ignore favicon.ico requests
                    if request.uri().path() == "/favicon.ico" {
                        return;
                    }

                    // add the client IP address custom header
                    request
                        .headers_mut()
                        .insert("X-Client-IP", client_address.to_string().parse().unwrap());

                    let uri = request.uri().path().to_string();

                    let request: load_balancer::request::Request =
                        load_balancer::request::Request::new(
                            uri,
                            client_address.to_string(),
                            request,
                        );

                    if state.lock().await.insert(request).await {
                        // request got added
                        let _ = state.lock().await.distribute().await;
                        return;
                    } else {
                        // request not added respond status 429 too many requests
                        send_error_response(429, &mut stream).await;
                        return;
                    }
                }
                return;
            });
        }
    }
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    eprintln!("Shutdown signal received...");
}

fn get_nodes() -> Vec<String> {
    // Load the .env file
    dotenv().ok();

    let mut nodes: Vec<String> = Vec::new();

    for (key, value) in env::vars() {
        if key.starts_with("NODE") {
            nodes.push(value);
        }
    }

    println!("Loaded nodes");

    nodes
}

async fn send_error_response(code: u64, stream: &mut TcpStream) {
    match code {
        429 => {
            let response_bytes =
                format!("HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\n\r\n",)
                    .into_bytes();

            let _ = stream.write_all(&response_bytes).await;
        }
        _ => {
            let response_bytes =
                format!("HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n",).into_bytes();

            let _ = stream.write_all(&response_bytes).await;
        }
    }
}
