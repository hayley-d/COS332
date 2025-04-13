use base64::engine::Engine as _;
use tokio::net::TcpStream;

pub async fn send_mail(
    subject: String,
    message: String,
    recipient: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // load the enviroment variables
    dotenv::dotenv().ok();

    let smtp_server: String = std::env::var("SMTP_SERVER")?;
    let port: u16 = std::env::var("SMTP_PORT")?.parse()?;
    let username: String = std::env::var("SMTP_USERNAME")?;
    let token: String = std::env::var("SMTP_TOKEN")?;

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
    send_command(&mut stream, "EHLO alarm.local\r\n", ResponseKind::Multi).await?;

    // Start TLS
    send_command(&mut stream, "STARTTLS\r\n", ResponseKind::Single).await?;

    // Upgrade to TLS
    let connector: tokio_native_tls::TlsConnector =
        tokio_native_tls::TlsConnector::from(match native_tls::TlsConnector::new() {
            Ok(tls) => tls,
            Err(_) => {
                eprintln!("Failed to esablish TLS connection");
                std::process::exit(1);
            }
        });

    let mut tls_stream = connector.connect(&smtp_server, stream).await?;

    // Send HELO again after TLS is established
    send_command(&mut tls_stream, "EHLO alarm.local\r\n", ResponseKind::Multi).await?;

    let auth_string: String = format!("\0{}\0{}", username, token);
    let auth_encoded = base64::engine::general_purpose::URL_SAFE.encode(auth_string);

    // Authenticate using AUTH PLAIN
    send_command(
        &mut tls_stream,
        &format!("AUTH PLAIN {}\r\n", auth_encoded),
        ResponseKind::Single,
    )
    .await?;

    // Specify the sender
    send_command(
        &mut tls_stream,
        &format!("MAIL FROM:<{}>\r\n", username),
        ResponseKind::Single,
    )
    .await?;

    // Specify the recipient
    send_command(
        &mut tls_stream,
        &format!("RCPT TO:<{}>\r\n", recipient),
        ResponseKind::Single,
    )
    .await?;

    // Start composing email
    send_command(&mut tls_stream, "DATA\r\n", ResponseKind::Single).await?;

    // Send the email
    let email_content: String = format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\n\r\n{}\r\n.\r\n",
        username, recipient, subject, message
    );
    send_command(&mut tls_stream, &email_content, ResponseKind::Single).await?;

    // Quit the session
    send_command(&mut tls_stream, "QUIT\r\n", ResponseKind::Single).await?;

    log::info!(target:"request_logger","Email send to {}",recipient);
    Ok(())
}

pub(crate) enum ResponseKind {
    Single,
    Multi,
}

async fn send_command<S>(
    stream: &mut S,
    command: &str,
    response_kind: ResponseKind,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: tokio::io::AsyncWriteExt + Unpin + tokio::io::AsyncReadExt,
{
    stream.write(command.as_bytes()).await?;
    stream.flush().await?;
    //println!("Sent command: {}", command);
    let response = match response_kind {
        ResponseKind::Single => read_single_response(stream).await?,
        _ => read_multiline_response(stream).await?,
    };

    let status_code: u16 = response[..3].parse()?;
    if !(200..=399).contains(&status_code) {
        return Err(format!("SMTP Error: {}", response.trim()).into());
    }

    Ok(())
}

async fn read_multiline_response<S>(stream: &mut S) -> Result<String, Box<dyn std::error::Error>>
where
    S: tokio::io::AsyncReadExt + Unpin,
{
    let mut buffer = [0u8; 1024];
    let mut response = String::new();

    loop {
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        let part = String::from_utf8_lossy(&buffer[..bytes_read]);
        response.push_str(&part);

        if part.contains("\r\n250 ") || part.trim_end().starts_with("250 ") {
            break;
        }
    }

    Ok(response)
}

async fn read_single_response<S>(stream: &mut S) -> Result<String, Box<dyn std::error::Error>>
where
    S: tokio::io::AsyncReadExt + Unpin,
{
    let mut buffer = [0u8; 1024];
    let bytes_read = stream.read(&mut buffer).await?;

    if bytes_read == 0 {
        return Err("Server closed connection".into());
    }

    let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
    Ok(response)
}
