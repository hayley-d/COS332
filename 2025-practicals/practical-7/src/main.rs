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
    let state: State = State::new("127.0.0.1", "hayleydod@proton.me", "YskUEuNu-zSRWtiNizzaxg");

    let mut stream: tokio::net::TcpStream =
        match tokio::net::TcpStream::connect((state.address, 1143)).await {
            Ok(s) => {
                println!("Connected to 127.0.0.1 on port 1143");
                s
            }
            Err(e) => {
                eprintln!("Failed to connect to SMTP sever: {}", e);
                std::process::exit(1);
            }
        };

    let mut greeting = vec![0u8; 1024];
    match stream.read(&mut greeting).await {
        Ok(bytes_read) => {
            let greeting_str = String::from_utf8_lossy(&greeting[..bytes_read]);
            println!("Server Greeting: {}", greeting_str);
        }
        Err(e) => {
            eprintln!("Failed to read server greeting: {}", e);
            std::process::exit(1);
        }
    }

    let login_command: String = format!("a001 LOGIN {} {}\r\n", state.username, state.password);
    let _ = stream.write_all(login_command.as_bytes()).await;

    println!("Successful Login");
    let mut greeting = vec![0u8; 1024];
    match stream.read(&mut greeting).await {
        Ok(bytes_read) => {
            let greeting_str = String::from_utf8_lossy(&greeting[..bytes_read]);
            println!("Server Login: {}", greeting_str);
        }
        Err(e) => {
            eprintln!("Failed to read server login: {}", e);
            std::process::exit(1);
        }
    }

    let select_command: &str = "a002 SELECT inbox\r\n";
    let _ = stream.write_all(select_command.as_bytes()).await;
    println!("Successful select command");
    let mut greeting = vec![0u8; 1024];
    match stream.read(&mut greeting).await {
        Ok(bytes_read) => {
            let greeting_str = String::from_utf8_lossy(&greeting[..bytes_read]);
            println!("Server Select: {}", greeting_str);
        }
        Err(e) => {
            eprintln!("Failed to read server Select: {}", e);
            std::process::exit(1);
        }
    }

    let fetch_command: &str = "a003 FETCH 1:* (BODY[HEADER.FIELDS (FROM SUBJECT SIZE)])\r\n";
    stream.write_all(fetch_command.as_bytes()).await?;
    println!("Successful Fetch command");

    let mut fetch = vec![0u8; 1024];

    let fetched_str = match stream.read(&mut fetch).await {
        Ok(bytes_read) => {
            let fetch_str = String::from_utf8_lossy(&greeting[..bytes_read]);
            println!("Server Select: {}", fetch_str);
            fetch_str
        }
        Err(e) => {
            eprintln!("Failed to read server Select: {}", e);
            std::process::exit(1);
        }
    };

    parse_imap_response(fetched_str.to_string());

    Ok(())
}

// Function to parse the IMAP response and extract email headers
fn parse_imap_response(response: String) {
    let lines: Vec<&str> = response.split("\r\n").collect();
    let mut current_email = String::new();

    for line in lines {
        if line.contains('*') {
            if !current_email.is_empty() {
                println!("{}", current_email);
            }

            current_email = String::new();
        }

        if line.starts_with("From:") {
            current_email.push_str(&format!("Sender: {}\n", &line[5..].trim()));
        } else if line.starts_with("Subject:") {
            current_email.push_str(&format!("Subject: {}\n", &line[8..].trim()));
        } else if line.starts_with("Size:") {
            current_email.push_str(&format!("Size: {}\n", &line[5..].trim()));
        }
    }

    if !current_email.is_empty() {
        println!("{}", current_email);
    }
}
