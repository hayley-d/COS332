use base64::engine::Engine as _;
use tokio::net::TcpStream;

pub async fn send_mail(
    _results: String,
    recipient: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // load the enviroment variables
    dotenv::dotenv().ok();

    let smtp_server: String = std::env::var("SMTP_SERVER")?;
    let port: u16 = std::env::var("SMTP_PORT")?.parse()?;
    let username: String = std::env::var("SMTP_USERNAME")?;
    let token: String = std::env::var("SMTP_TOKEN")?;

    let subject: &str = "Test Results";
    let body: &str = "Congratulations! You scored 95/100";

    // Connect to SMTP server
    println!("Connecting to {} on port {}", smtp_server, port);
    let mut stream: TcpStream = match TcpStream::connect((smtp_server.clone(), port)).await {
        Ok(s) => {
            println!("Connected to {smtp_server} on port {port}");
            s
        }
        Err(e) => {
            eprintln!("Failed to connect to SMTP sever: {}", e);
            return Err(Box::new(e));
        }
    };

    // Send HELO
    send_command(&mut stream, "HELO 192.168.101.111\r\n").await?;

    // Start TLS
    send_command(&mut stream, "STARTTLS\r\n").await?;

    let connector: tokio_native_tls::TlsConnector =
        tokio_native_tls::TlsConnector::from(native_tls::TlsConnector::new()?);
    let mut tls_stream = connector.connect(&smtp_server, stream).await?;

    // Send HELO again after TLS is established
    send_command(&mut tls_stream, "HELO 192.168.101.111\r\n").await?;

    let auth_string: String = format!("\0{}\0{}", username, token);
    let auth_encoded = base64::engine::general_purpose::URL_SAFE.encode(auth_string);

    // Authenticate using AUTH PLAIN
    send_command(&mut tls_stream, &format!("AUTH PLAIN {}\r\n", auth_encoded)).await?;

    // Specify the sender
    send_command(&mut tls_stream, &format!("MAIL FROM:<{}>\r\n", username)).await?;

    // Specify the recipient
    send_command(&mut tls_stream, &format!("RCPT TO:<{}>\r\n", recipient)).await?;

    // Start composing email
    send_command(&mut tls_stream, "DATA\r\n").await?;

    // Send the email
    let email_content: String = format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\n\r\n{}\r\n.\r\n",
        username, recipient, subject, body
    );
    send_command(&mut tls_stream, &email_content).await?;

    // Quit the session
    send_command(&mut tls_stream, "QUIT\r\n").await?;

    log::info!(target:"request_logger","Email send to {}",recipient);
    Ok(())
}

async fn send_command<S>(stream: &mut S, command: &str) -> Result<(), Box<dyn std::error::Error>>
where
    S: tokio::io::AsyncWriteExt + Unpin + tokio::io::AsyncReadExt,
{
    stream.write(command.as_bytes()).await?;
    stream.flush().await?;
    println!("Sent command: {}", command);

    read_response(stream, &mut Vec::new()).await?;
    Ok(())
}

async fn read_response<S>(
    stream: &mut S,
    _buf: &mut [u8],
) -> Result<String, Box<dyn std::error::Error>>
where
    S: tokio::io::AsyncReadExt + Unpin,
{
    let mut buffer: [u8; 1024] = [0; 1024];
    let bytes_read = stream.read(&mut buffer).await?;
    let response: String = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

    println!("MAIL SERVER RESPONSE: {}", response);
    let status_code: u16 = response[..3].parse()?;

    println!("Server: {}", response.trim());
    match status_code {
        200..=399 => Ok(response),
        _ => {
            log::error!(target:"error_logger","SMTP Error {}",response.trim());
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("SMTP Error {}", response.trim()),
            )));
        }
    }
}
