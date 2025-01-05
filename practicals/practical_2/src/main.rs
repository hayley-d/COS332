use std::collections::HashSet;

use libc::*;
use practical_2::connection::{accept_connection, create_raw_socket, handle_telnet_connection};
use practical_2::question::Question;
use rand::Rng;
use std::collections::HashSet;
use std::ffi::CString;
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddrV4};
use std::ptr;
use std::str;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt};

use std::error::Error;

/// Ask for username and password to use telnet services,
/// make sure no dangerous commands are contained such as rm -rf, rm
///

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
    let server_fd = create_raw_socket(port);

    loop {
        unsafe {
            let mut client_address: sockaddr_in = std::mem::zeroed::<sockaddr_in>();
            let mut address_len: u32 = std::mem::size_of::<sockaddr_in>() as u32;

            let client_fd = accept(
                server_fd,
                &mut client_address as *mut sockaddr_in as *mut sockaddr,
                &mut address_len,
            );

            if client_fd < 0 {
                eprintln!("Failed to accept connection");
                continue;
            }

            let question_clone: Arc<Vec<Question>> = Arc::clone(&questions);
            std::thread::spawn(move || {
                handle_telnet_connection(client_fd, question_clone);
            });
        }
    }
}

async fn game() -> () {
    loop {
        println!("Do you want a question? (y/n): ");

        let mut stdin = io::BufReader::new(io::stdin());
        let mut input: String = String::new();

        match stdin.read_line(&mut input).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        }

        match input.as_str().trim() {
            "y" => {
                let random = rand::thread_rng().gen_range(0..questions.len());
                println!("Enter the number of the correct answer (e.g. 1 or 1,2 or leave blank for none): ");
                println!("{}", questions[random].print());

                let mut answer_input: String = String::new();
                match stdin.read_line(&mut answer_input).await {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        std::process::exit(1);
                    }
                }

                let answers: HashSet<usize> = answer_input
                    .trim()
                    .split(',')
                    .filter_map(|s| s.parse::<usize>().ok().map(|n| n - 1))
                    .collect();

                questions[random].check_answer(answers.into_iter().collect());
                continue;
            }
            "n" => {
                break;
            }
            _ => {
                eprintln!("Please only enter y or n");
                continue;
            }
        }
    }
}
