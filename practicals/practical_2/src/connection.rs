use core::str;
use libc::*;
use rand::Rng;
use std::error::Error;
use std::sync::Arc;

use crate::question::Question;

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

        println!("Server is listening on port {}", port);
        return Ok(socket_fd);
    }
}

pub fn handle_telnet_connection(
    client_fd: i32,
    questions: Arc<Vec<Question>>,
) -> Result<(), Box<dyn Error>> {
    unsafe {
        let mut buffer: [u8; 1024] = [0; 1024];
        let welcome_msg: &str = "Welcome to the Telnet server!";

        // write the welcome message to the client
        write(
            client_fd,
            welcome_msg.as_ptr() as *const c_void,
            welcome_msg.len(),
        );

        loop {
            let welcome_msg: &str = "Do you want a question? (y/n): ";
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

            let input: &str = str::from_utf8(&buffer[0..bytes_read as usize])
                .unwrap_or_default()
                .trim();

            match input {
                "y" => {
                    let random: usize = rand::thread_rng().gen_range(0..questions.len());
                    let question: &Question = &questions[random];
                    let question_txt: String = format!(
                        "{}\nEnter the correct answer(s) (e.g., 1 or 1,2 or leave blank): ",
                        question.print()
                    );

                    write(
                        client_fd,
                        question_txt.as_ptr() as *const c_void,
                        question_txt.len(),
                    );

                    let bytes_read: usize =
                        read(client_fd, buffer.as_mut_ptr() as *mut c_void, buffer.len()) as usize;

                    if bytes_read <= 0 {
                        break;
                    }

                    let answer_input: &str = str::from_utf8(&buffer[0..bytes_read])
                        .unwrap_or_default()
                        .trim();

                    let answers: Vec<usize> = answer_input
                        .split(',')
                        .filter_map(|s| s.parse::<usize>().ok().map(|n| n - 1))
                        .collect();

                    let answers = question.check_answer(answers);

                    write(client_fd, answers.as_ptr() as *const c_void, answers.len());
                }
                "n" => {
                    let goodbye_msg: &str = "Goodbye!\n";
                    write(
                        client_fd,
                        goodbye_msg.as_ptr() as *const c_void,
                        goodbye_msg.len(),
                    );
                    break;
                }
                _ => {
                    let error_msg: &str = "Invalid input. Please enter 'y' or 'n'.\n";
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
