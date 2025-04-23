use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?; // Listen on POP3 default port

    println!("POP3 server running on 127.0.0.1:7878");

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let _ = socket.write_all(b"+OK POP3 server ready\r\n").await; // Greet the client

            let mut buffer = vec![0; 1024]; // Buffer for reading commands

            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let command = String::from_utf8_lossy(&buffer[..n]);

                        println!("Received command: {}", command);

                        if command.starts_with("USER") {
                            // Handle USER command
                            let _ = socket.write_all(b"+OK User accepted\r\n").await;
                        } else if command.starts_with("PASS") {
                            // Handle PASS command
                            let _ = socket.write_all(b"+OK Password accepted\r\n").await;
                        } else if command.starts_with("LIST") {
                            // Simulate listing messages
                            let _ = socket
                                .write_all(b"+OK 2 messages\r\n1 1000\r\n2 2000\r\n")
                                .await;
                        } else if command.starts_with("RETR") {
                            // Simulate retrieving a message
                            let _ = socket.write_all(b"+OK Message retrieved\r\nSubject: Test\r\nFrom: example@example.com\r\n\r\nHello, world!\r\n").await;
                        } else if command.starts_with("DELE") {
                            // Simulate deleting a message
                            let _ = socket.write_all(b"+OK Message deleted\r\n").await;
                        } else if command.starts_with("QUIT") {
                            // Handle QUIT command
                            let _ = socket.write_all(b"+OK Goodbye\r\n").await;
                            break;
                        } else {
                            // Handle unrecognized commands
                            let _ = socket.write_all(b"-ERR Unknown command\r\n").await;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}
