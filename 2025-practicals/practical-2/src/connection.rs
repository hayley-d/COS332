use crate::database::Database;
use core::str;
use libc::*;
use rand::Rng;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn create_raw_socket(port: u16) -> Result<i32, Box<dyn Error>> {
    unsafe {
        // Create a socket
        // AF_INET specifies the IPv4 address fam
        // SOCK_STREAM indicates that the socket will use TCP
        // 0 is default for TCP
        let socket_fd = socket(AF_INET, SOCK_STREAM, 0);

        if socket_fd < 0 {
            eprintln!("Failed to create socket");
            std::process::exit(1);
        }

        // Set socket options
        let option_val: i32 = 1;
        if setsockopt(
            socket_fd,
            SOL_SOCKET,
            SO_REUSEADDR,
            &option_val as *const _ as *const c_void,
            std::mem::size_of_val(&option_val) as u32,
        ) < 0
        {
            eprintln!("Failed to set socket options");
            std::process::exit(1);
        }

        // Bind socket to address
        let address = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(port),
            sin_addr: in_addr { s_addr: INADDR_ANY },
            sin_zero: [0; 8],
        };

        if bind(
            socket_fd,
            &address as *const sockaddr_in as *const sockaddr,
            std::mem::size_of::<sockaddr_in>() as u32,
        ) < 0
        {
            eprintln!("Failed to bind socket to address");
            std::process::exit(1);
        }

        // Start listening at address
        if listen(socket_fd, 128) < 0 {
            eprintln!("Failed to listen on socket");
            std::process::exit(1);
        }

        println!("Server is listening on port {}", port);
        return Ok(socket_fd);
    }
}

pub async fn handle_telnet_connection(
    client_fd: i32,
    database: Arc<Mutex<Database>>,
) -> Result<(), Box<dyn Error>> {
    unsafe {
        let mut buffer: [u8; 1024] = [0; 1024];
        let welcome_msg: &str = "Welcome to the Telnet Friend Database!\n";

        write(
            client_fd,
            welcome_msg.as_ptr() as *const c_void,
            welcome_msg.len(),
        );

        loop {
            let welcome_msg: &str =
                "Available Commands:\n1) Add <name> <phone>\n2) Get <name>\n3) Delete <name>\nEXIT\n";
            write(
                client_fd,
                welcome_msg.as_ptr() as *const c_void,
                welcome_msg.len(),
            );
            // Read the input from the client
            let bytes_read = read(client_fd, buffer.as_mut_ptr() as *mut c_void, buffer.len());
            if bytes_read <= 0 {
                break;
            }

            let mut input = str::from_utf8(&buffer[0..bytes_read as usize])
                .unwrap_or_default()
                .trim()
                .split_whitespace();

            let command = input.next();

            match command {
                Some("ADD") | Some("add") => {
                    if let (Some(name), Some(phone)) = (input.next(), input.next()) {
                        match database.lock().await.add_friend(name, phone) {
                            Ok(_) => {
                                let response: String =
                                    format!("Added {} with number {}", name, phone);
                                write(
                                    client_fd,
                                    response.as_ptr() as *const c_void,
                                    response.len(),
                                );
                            }
                            Err(e) => {
                                let response: String =
                                    format!("Error adding friend to the database:{:?}", e);
                                write(
                                    client_fd,
                                    response.as_ptr() as *const c_void,
                                    response.len(),
                                );
                            }
                        }
                    }
                }
                Some("GET") | Some("get") => {
                    if let Some(name) = input.next() {
                        match database.lock().await.get_friend(name) {
                            Ok(Some(phone)) => {
                                let response: String = format!("{} : {}", name, phone);
                                write(
                                    client_fd,
                                    response.as_ptr() as *const c_void,
                                    response.len(),
                                );
                            }
                            Ok(None) => {
                                let response: String = String::from("Error friend not found");
                                write(
                                    client_fd,
                                    response.as_ptr() as *const c_void,
                                    response.len(),
                                );
                            }
                            Err(e) => {
                                let response: String = format!("Error retrieving friend:{:?}", e);
                                write(
                                    client_fd,
                                    response.as_ptr() as *const c_void,
                                    response.len(),
                                );
                            }
                        }
                    }
                }
                Some("DELETE") | Some("delete") => {
                    if let Some(name) = input.next() {
                        match database.lock().await.delete_friend(name) {
                            Ok(_) => {
                                let response: String =
                                    format!("{} has been removed from the database", name);
                                write(
                                    client_fd,
                                    response.as_ptr() as *const c_void,
                                    response.len(),
                                );
                            }
                            Err(e) => {
                                let response: String = format!("Error removing friend:{:?}", e);
                                write(
                                    client_fd,
                                    response.as_ptr() as *const c_void,
                                    response.len(),
                                );
                            }
                        }
                    }
                }
                Some("EXIT") | Some("exit") => {
                    let goodbye_msg: &str = "Goodbye!\n";
                    write(
                        client_fd,
                        goodbye_msg.as_ptr() as *const c_void,
                        goodbye_msg.len(),
                    );
                    break;
                }
                _ => {
                    let error_msg: &str = "Invalid input. Please enter a valid command.\n";
                    write(
                        client_fd,
                        error_msg.as_ptr() as *const c_void,
                        error_msg.len(),
                    );
                }
            }
        }
        close(client_fd);
    }
    return Ok(());
}
