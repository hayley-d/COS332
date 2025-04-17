use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct State<'a> {
    address: &'a str,
    username: &'a str,
    password: &'a str,
}

impl<'a> State<'a> {
    pub fn new(address: &'a str, username: &'a str, password: &'a str) -> State<'a> {
        State {
            address,
            username,
            password,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state: State = State::new(
        "127.0.0.1:1143",
        "hayleydod@proton.me",
        "YskUEuNu-zSRWtiNizzaxg",
    );

    let mut stream = tokio::net::TcpStream::connect(state.address).await?;
    println!("Successful Connection to 1143");

    let login_command: String = format!("a001 LOGIN {} {}\r\n", state.username, state.password);
    stream.write_all(login_command.as_bytes()).await?;
    println!("Successful Login");

    let select_command: &str = "a002 SELECT inbox\r\n";
    stream.write_all(select_command.as_bytes()).await?;
    println!("Successful select command");

    let fetch_command: &str = "a003 FETCH 1:* (BODY[HEADER.FIELDS (FROM SUBJECT SIZE)])\r\n";
    stream.write_all(fetch_command.as_bytes()).await?;
    println!("Successful Fetch command");

    let mut response: Vec<u8> = Vec::new();
    stream.read(&mut response).await?;

    println!("IMAP Response: {}", String::from_utf8_lossy(&response));
    Ok(())
}
