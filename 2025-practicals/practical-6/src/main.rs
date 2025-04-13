use practical_6::server::set_up_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = set_up_server().await;

    Ok(())
}
