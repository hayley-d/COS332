use core::str;

use practical_2::question::Question;
use rand::Rng;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    game().await;
}

async fn game() {
    let questions: Vec<Question> = Question::parse_file().await;

    loop {
        print!("Do you want a question? (y/n): ");

        let _ = io::stdout().flush().await;

        let mut input: Vec<u8> = Vec::new();

        match io::stdin().read_to_end(&mut input).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        }

        let input: String = match str::from_utf8(&input) {
            Ok(s) => s.trim().to_lowercase(),
            Err(e) => {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        };

        match input.as_str() {
            "y" => {
                let random = rand::thread_rng().gen_range(0..questions.len());
                println!("{}", questions[random].print());
                println!("Enter the number of the correct answer (e.g. 1 or 1,2 or leave blank for none)");
                let _ = io::stdout().flush().await;

                let mut answer_input: Vec<u8> = Vec::new();
                match io::stdin().read_to_end(&mut answer_input).await {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        std::process::exit(1);
                    }
                }

                let answers: Vec<usize> = match str::from_utf8(&answer_input) {
                    Ok(s) => s
                        .trim()
                        .split(',')
                        .filter_map(|s| s.parse::<usize>().ok().map(|n| n - 1))
                        .collect(),
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        std::process::exit(1);
                    }
                };

                questions[random].check_answer(answers);
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
