use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Bytes as httpBytes;
use hyper::server::conn::http1;
use hyper::service::{service_fn, Service};
use hyper::{Request, Response, Uri};
use hyper_util::rt::TokioIo;
use load_balancer::load_balancer::load_balancer::LoadBalancer;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

const RATELIMITERADDRESS: &str = "http://127.0.0.1:7879";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // listend on port 3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener: TcpListener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(_) => {
            eprintln!("Failed to bind to socket");
            std::process::exit(1);
        }
    };

    println!("Listening on http://{}", addr);

    let nodes: Vec<String> = vec![
        String::from("http://127.0.0.1:7878"),
        String::from("http://127.0.0.1:7879"),
    ];

    let state: LoadBalancer = match LoadBalancer::new(2, &mut nodes).await {
        Ok(s) => s,
        Err(_) => std::process::exit(1),
    };

    let state: Arc<Mutex<LoadBalancer>> = Arc::new(Mutex::new(state));

    let mut http = http1::Builder::new();
    let mut http = http.preserve_header_case(true).title_case_headers(true);

    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    let mut signal = std::pin::pin!(shutdown_signal());

    loop {
        let state = state.clone();
        tokio::select! {
            Ok((stream,address)) = listener.accept() => {
                let io = TokioIo::new(stream);

                let state_clone = state.clone();

                let connection = http.serve_connection(
                    io,
                    service_fn(move |req| proxy(req, address, state_clone.clone())),
                );

                let fut = graceful.watch(connection);

                tokio::spawn(async move{
                    if let Err(e) = fut.await {
                        eprintln!("Error serving connecion");
                    }
                });
            },
            _ = &mut signal => {
                eprintln!("graceful shutdown signal recieved");
                break;
            }
        }
    }

    Ok(())
}

impl Service<hyper::Request<hyper::body::Incoming>> for LoadBalancer {
    type Response = Response<Full<Bytes>>;

    type Error = hyper::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<hyper::body::Incoming>) -> Self::Future {
        fn make_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
        }

        todo!()
    }
}

async fn proxy(
    req: Request<hyper::body::Incoming>,
    client_address: SocketAddr,
    state: Arc<Mutex<LoadBalancer>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let uri = req.uri().path().to_string();
    let request: load_balancer::request::Request =
        load_balancer::request::Request::new(uri, client_address.to_string());

    if req.uri().path() != "/favicon.ico" {
        let uri = req.uri().to_string();
    } else {
        let res = make_response("Not found".into());
        return Box::pin(async { res });
    }

    if state.lock().await.insert(request).await {
        // request got added
    } else {
        // request not added respond status 429 too many requests
        return Ok(Response::builder()
            .status(429)
            .body(BoxBody::new(Ok(Bytes::from("Too Many Requests"))))
            .unwrap());
    }

    todo!()
}

// Service that define how the server responds to the request
async fn handle_request(
    request: Request<hyper::body::Incoming>,
    client_address: Option<SocketAddr>,
    state: Arc<Mutex<LoadBalancer>>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let target_uri: String = get_target_uri(&request).to_string();

    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

// Extract the target URI
fn get_target_uri(req: &Request<hyper::body::Incoming>) -> Uri {
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("");

    let target_uri = format!("{}", path_and_query);

    target_uri.parse().unwrap()
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
