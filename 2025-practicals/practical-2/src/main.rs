use libc::*;
use practical_2::connection::{create_raw_socket, handle_telnet_connection};
use practical_2::database::Database;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

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

    let database: Arc<Mutex<Database>> = Arc::new(Mutex::new(
        Database::new("friends.db").expect("Failed to initialize database"),
    ));

    let server_fd = create_raw_socket(port).unwrap();

    // Limit the amout of async connections to 5
    let semaphore = Arc::new(Semaphore::new(5));

    loop {
        let client_fd;
        let permit = semaphore.acquire().await.unwrap();
        println!(
            "Current concurrent connections: {}",
            5 - semaphore.available_permits()
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

        let db_clone: Arc<Mutex<Database>> = Arc::clone(&database);

        tokio::spawn(async move {
            let _ = handle_telnet_connection(client_fd, db_clone).await;
        });
        drop(permit);
    }
}
