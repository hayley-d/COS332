use tokio::fs;

pub struct Question {
    question: String,
    options: Vec<String>,
    answers: Vec<usize>,
}

impl Question {
    pub fn new(question: String, options: Vec<String>, answers: Vec<usize>) -> Self {
        return Question {
            question,
            options,
            answers,
        };
    }

    pub fn print(&self) -> String {
        let mut output: String = format!("\033[1;34mQuestion: {}\x1b[0m", self.question);
        for (i, option) in self.options.iter().enumerate() {
            output.push_str(format!("({}) {}", i, option).as_str());
        }

        return output;
    }

    pub async fn parse_file() -> Vec<Question> {
        let file = match fs::read_to_string("file.txt").await {
            Ok(f) => f,
            Err(_) => {
                eprintln!("Error reading file");
                std::process::exit(1);
            }
        };
        println!("{}", file);
        return Vec::new();
    }
}
