use std::collections::HashSet;

use practical_2::connection::{
    accept_connection, create_listener, create_raw_listener, create_socket, handle_connection, handle_telnet_connection
};
use practical_2::question::Question;
use rand::Rng;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
unsafe async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    unsafe {
        let listener = create_raw_listener(port)?;
        println!("Server running on port {}",port);

        loop {
            let client_fd = accept_connection(listener)?;

                handle_telnet_connection(client_fd)?;    
        }
    }

    Ok(())
}

async fn game() -> () {
    let questions: Vec<Question> = Question::parse_file().await;

    loop {
        println!("Do you want a question? (y/n): ");
        //io::stdout().flush().await.unwrap();

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
                //io::stdout().flush().await.unwrap();

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
