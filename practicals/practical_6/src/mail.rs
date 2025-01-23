use tokio::net::TcpStream;

pub async fn send_mail(
    results: String,
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
    let mut stream: TcpStream = TcpStream::connect((smtp_server.clone(), port)).await?;

    // Send EHLO
    send_command(&mut stream, "EHLO localhost").await?;

    // Start TLS
    send_command(&mut stream, "STARTTLS").await?;

    let connector: tokio_native_tls::TlsConnector =
        tokio_native_tls::TlsConnector::from(native_tls::TlsConnector::new()?);
    let mut tls_stream = connector.connect(&smtp_server, stream).await?;

    // Send EHLO again after TLS is established
    send_command(&mut tls_stream, "EHLO localhost").await?;

    let auth_string: String = format!("\0{}\0{}", username, token);
    let auth_encoded = base64::encode(auth_string);

    // Authenticate using AUTH PLAIN
    send_command(&mut tls_stream, &format!("AUTH PLAIN {}", auth_encoded)).await?;

    // Specify the sender
    send_command(&mut tls_stream, &format!("MAIL FROM: {}", username)).await?;

    // Specify the recipient
    send_command(&mut tls_stream, &format!("RCPT TO:<{}>", recipient)).await?;

    // Start composing email
    send_command(&mut tls_stream, "DATA").await?;

    // Send the email
    let email_content: String = format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\n\r\n{}\r\n.",
        username, recipient, subject, body
    );
    send_command(&mut tls_stream, &email_content).await?;

    // Quit the session
    send_command(&mut tls_stream, "QUIT").await?;

    log::info!(target:"request_logger","Email send to {}",recipient);
    Ok(())
}

async fn send_command<S>(stream: &mut S, command: &str) -> Result<(), Box<dyn std::error::Error>>
where
    S: tokio::io::AsyncWriteExt + Unpin + tokio::io::AsyncReadExt,
{
    stream.write_all(command.as_bytes()).await?;
    stream.write_all(b"\r\n").await?;
    stream.flush().await?;

    read_response(stream, &mut Vec::new()).await?;
    Ok(())
}

async fn read_response<S>(
    stream: &mut S,
    buffer: &mut [u8],
) -> Result<String, Box<dyn std::error::Error>>
where
    S: tokio::io::AsyncReadExt + Unpin,
{
    let n = stream.read(buffer).await?;
    let response: String = String::from_utf8_lossy(&buffer[..n]).to_string();
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
