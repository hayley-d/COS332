use libc::*;
use practical_4::connection::{create_raw_socket, handle_telnet_connection};
use practical_4::question::Question;
use std::sync::Arc;
use tokio::sync::Semaphore;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let port: u16 = match std::env::args().collect::<Vec<String>>().get(1) {
        Some(p) => match p.parse::<u16>() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Error: No port provided in command line arguments");
                std::process::exit(1);
            }
        },
        None => {
            eprintln!("Error: No port provided in command line arguments");
            std::process::exit(1);
        }
    };

    let questions: Arc<Vec<Question>> = Arc::new(Question::parse_file().await);
    let server_fd = create_raw_socket(port).unwrap();

    // Limit the amout of async connections to 5
    let semaphore = Arc::new(Semaphore::new(5));

    loop {
        let client_fd;
        let permit = semaphore.acquire().await.unwrap();
        println!(
            "Current concurrent connections: {}",
            semaphore.available_permits() - 5
        );

        unsafe {
            let mut client_address: sockaddr_in = std::mem::zeroed::<sockaddr_in>();
            let mut address_len: u32 = std::mem::size_of::<sockaddr_in>() as u32;

            client_fd = accept(
                server_fd,
                &mut client_address as *mut sockaddr_in as *mut sockaddr,
                &mut address_len,
            );

            if client_fd < 0 {
                eprintln!("Failed to accept connection");
                continue;
            }
        }
        let question_clone: Arc<Vec<Question>> = Arc::clone(&questions);
        tokio::spawn(async move {
            let _ = handle_telnet_connection(client_fd, question_clone);
        });
        drop(permit);
        println!("Connection dropped");
    }
}
