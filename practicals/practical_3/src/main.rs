use practical_3::connection::start_server;

#[tokio::main]
async fn main() {
    let _ = start_server(7878).await;
}
