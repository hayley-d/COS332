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

    /*match stream.write_all("STARTTLS\r\n".as_bytes()).await {
        Ok(_) => {
            println!("STARTTLS command sent");
        }
        Err(e) => {
            eprintln!("Failed to send STARTTLS command: {}", e);
            std::process::exit(1);
        }
    }

    let _ = stream.flush();

    let connector: tokio_native_tls::TlsConnector =
        tokio_native_tls::TlsConnector::from(match native_tls::TlsConnector::new() {
            Ok(tls) => tls,
            Err(_) => {
                eprintln!("Failed to esablish TLS Connector");
                std::process::exit(1);
            }
        });

    let connector = tokio_native_tls::TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()?,
    );

    let mut tls_stream = match connector.connect("localhost", stream).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to initialize TLS: {:?}", e);
            std::process::exit(1);
        }
    };*/

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
    let mut greeting = vec![0u8; 1024];
    match stream.read(&mut greeting).await {
        Ok(bytes_read) => {
            let greeting_str = String::from_utf8_lossy(&greeting[..bytes_read]);
            println!("Server Fetch: {}", greeting_str);
        }
        Err(e) => {
            eprintln!("Failed to read server Fetch: {}", e);
            std::process::exit(1);
        }
    }

    let mut response: Vec<u8> = Vec::new();
    stream.read(&mut response).await?;

    parse_imap_response(&response);

    Ok(())
}

// Function to parse the IMAP response and extract email headers
fn parse_imap_response(response: &[u8]) {
    let response_str: std::borrow::Cow<'_, str> = String::from_utf8_lossy(response);
    let lines: Vec<&str> = response_str.split("\r\n").collect();

    let mut current_email = String::new();

    // Process each line of the response
    for line in lines {
        // If the line starts with "*", it's a new email entry
        if line.starts_with('*') {
            // Print the current email details (previous email data)
            if !current_email.is_empty() {
                println!("{}", current_email);
            }

            // Start a new email entry
            current_email = String::new();
        }

        // Extract fields like "From", "Subject", and "Size"
        if line.starts_with("From:") {
            current_email.push_str(&format!("Sender: {}\n", &line[5..].trim()));
        } else if line.starts_with("Subject:") {
            current_email.push_str(&format!("Subject: {}\n", &line[8..].trim()));
        } else if line.starts_with("Size:") {
            current_email.push_str(&format!("Size: {}\n", &line[5..].trim()));
        }
    }

    // Print the last email entry (in case the last email is missing the newline)
    if !current_email.is_empty() {
        println!("{}", current_email);
    }
}
