use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use load_balancer::load_balancer::load_balancer::LoadBalancer;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_util::codec::Framed;

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

    let state: LoadBalancer = match LoadBalancer::new(2, &mut nodes).await {
        Ok(s) => s,
        Err(_) => std::process::exit(1),
    };

    let state: Arc<Mutex<LoadBalancer>> = Arc::new(Mutex::new(state));

    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    let mut signal = std::pin::pin!(shutdown_signal());

    loop {
        let state = state.clone();
        tokio::select! {
            Ok((stream,address)) = listener.accept() => {
                let state_clone = state.clone();

                tokio::spawn(async move{
                    let mut buffer: [u8;4096] = [0;u8];
                    let mut transport = Framed::new(stream,Http);
                    while let Some(request) = transport.next().await {
                        match request {
                            Ok(request) => {
                            },
                            Err(_) => {
                                return;
                            }
                        }
                    }
                    if let Err(_) = reverse_porxy(address,state_clone.clone()).await {
                        eprintln!("Error serving connecion");
                    }
                });

                todo!();
            },
            _ = &mut signal => {
                eprintln!("graceful shutdown signal recieved");
                break;
            }
        }
    }

    // Now start the shutdown and wait for them to complete
    tokio::select! {
        _ = graceful.shutdown() => {
            eprintln!("all connections gracefully closed");
        },
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            eprintln!("timed out wait for all connections to close");
        }
    }

    Ok(())
}

async fn reverse_proxy(
    req: http::Request<()>,
    client_address: SocketAddr,
    client_stream: TcpStream,
    state: Arc<Mutex<LoadBalancer>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ignore favicon.ico requests
    if req.uri().path() == "/favicon.ico" {
        return Ok(());
    }

    // add the client IP address custom header
    req.headers_mut()
        .insert("X-Client-IP", client_address.to_string().parse().unwrap());

    let uri = req.uri().path().to_string();

    let request: load_balancer::request::Request =
        load_balancer::request::Request::new(uri, client_address.to_string(), req);

    if state.lock().await.insert(request).await {
        // request got added
        let _ = state.lock().await.distribute().await;
    } else {
        // request not added respond status 429 too many requests

        return Ok(Response::builder()
            .status(429)
            .body(Full::new(Bytes::from("Too Many Request")))
            .unwrap());
    }

    todo!()
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
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
