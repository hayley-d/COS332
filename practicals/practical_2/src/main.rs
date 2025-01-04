use std::collections::HashSet;

use practical_2::question::Question;
use rand::Rng;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    println!("Hello");
    let _ = game().await;
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
