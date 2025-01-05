use libc::*;
use std::error::Error;
use std::ffi::CString;
use std::os::fd::RawFd;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub fn create_raw_socket(port: u16) -> Result<i32, Box<dyn Error>> {
    unsafe {
        // Create socket
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

        return Ok(socket_fd);
    }
}

// Handle a Telnet connection
pub async fn handle_connection(socket: &mut TcpStream) {
    let mut buffer = vec![0u8; 1024];

    // Welcome message
    let welcome_message = "Welcome to the Telnet server!\n";

    if let Err(e) = socket.write_all(welcome_message.as_bytes()).await {
        eprintln!("Error writing to socket: {}", e);
        return;
    }

    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[0..n]);

                if message.contains("IAC") {}
            }
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                break;
            }
        }
    }
}

pub unsafe fn accept_connection(listener_fd: RawFd) -> io::Result<RawFd> {
    let mut client_addr = sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: 0,
        sin_addr: libc::in_addr { s_addr: 0 },
        sin_zero: [0; 8],
    };
    let mut client_addr_len = std::mem::size_of::<sockaddr_in>() as u32;

    let client_fd = accept(
        listener_fd,
        &mut client_addr as *mut _ as *mut libc::sockaddr,
        &mut client_addr_len,
    );
    if client_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    println!("Client connected!");

    Ok(client_fd)
}

pub unsafe fn handle_telnet_connection(client_fd: RawFd) -> io::Result<()> {
    let mut buffer = [0u8; 1024];

    // Send a welcome message
    let welcome_message = "Welcome to the game! Let's start!\n";
    write(
        client_fd,
        welcome_message.as_bytes().as_ptr() as *const libc::c_void,
        welcome_message.len() as usize,
    );

    loop {
        // Ask if they want to play
        let question_prompt = "Do you want a question? (y/n): ";
        write(
            client_fd,
            question_prompt.as_ptr() as *const libc::c_void,
            question_prompt.len() as usize,
        );

        // Read response from client
        let n = read(
            client_fd,
            buffer.as_mut_ptr() as *mut libc::c_void,
            buffer.len() as usize,
        );
        if n <= 0 {
            eprintln!("Error or client closed the connection.");
            close(client_fd);
            break;
        }
        let message = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const libc::c_char);
        let input = message.to_string_lossy().trim().to_lowercase();

        match input.as_str() {
            "y" => {
                // For now, just simulate a question and answer check
                let question = "What is 2 + 2?";
                let options = "\n1. 3\n2. 4\n3. 5\n4. 6\n";
                let correct_answer = "2"; // Correct answer is 4

                // Send question and options
                write(
                    client_fd,
                    question.as_bytes().as_ptr() as *const libc::c_void,
                    question.len() as usize,
                );
                write(
                    client_fd,
                    options.as_bytes().as_ptr() as *const libc::c_void,
                    options.len() as usize,
                );

                // Ask for the answer
                let answer_prompt = "Enter your answer (e.g. 2 for answer 2): ";
                write(
                    client_fd,
                    answer_prompt.as_ptr() as *const libc::c_void,
                    answer_prompt.len() as usize,
                );

                // Read answer from client
                let n = read(
                    client_fd,
                    buffer.as_mut_ptr() as *mut libc::c_void,
                    buffer.len() as usize,
                );
                if n <= 0 {
                    eprintln!("Error reading answer.");
                    close(client_fd);
                    break;
                }
                let message = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const libc::c_char);
                let answer = message.to_string_lossy().trim();

                if answer == correct_answer {
                    let correct_message = "Correct!\n";
                    write(
                        client_fd,
                        correct_message.as_ptr() as *const libc::c_void,
                        correct_message.len() as usize,
                    );
                } else {
                    let incorrect_message = "Incorrect.\n";
                    write(
                        client_fd,
                        incorrect_message.as_ptr() as *const libc::c_void,
                        incorrect_message.len() as usize,
                    );
                }
            }
            "n" => {
                let goodbye_message = "Goodbye!\n";
                write(
                    client_fd,
                    goodbye_message.as_ptr() as *const libc::c_void,
                    goodbye_message.len() as usize,
                );
                close(client_fd);
                break;
            }
            _ => {
                let invalid_input_message = "Invalid input. Please enter 'y' or 'n'.\n";
                write(
                    client_fd,
                    invalid_input_message.as_ptr() as *const libc::c_void,
                    invalid_input_message.len() as usize,
                );
            }
        }
    }

    Ok(())
}
