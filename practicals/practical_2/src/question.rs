use tokio::fs;

pub struct Question {
    question: String,
    options: Vec<String>,
    answers: Vec<usize>,
}

impl Question {
    pub fn new(question: String) -> Self {
        return Question {
            question,
            options: Vec::new(),
            answers: Vec::new(),
        };
    }

    pub fn add_option(&mut self, option: String, is_answer: bool) {
        self.options.push(option);
        if is_answer {
            self.answers.push(self.options.len() - 1);
        }
    }

    pub fn print(&self) -> String {
        let mut output = format!("\x1b[1;34mQuestion: {}\x1b[0m\n", self.question);
        for (i, option) in self.options.iter().enumerate() {
            let formatted_option = format!("({}) {}\n", i + 1, option);
            output.push_str(formatted_option.as_str());
        }
        return output;
    }

    pub fn check_answer(&self, answers: Vec<usize>) -> String {
        let mut output: String = String::new();
        if answers != self.answers {
            output.push_str(
                format!("\x1b[1;31mIncorrect\x1b[0m the question answers are:\n").as_str(),
            );

            if self.answers.is_empty() {
                output.push_str(format!("\x1b[1;31mNo correct answers\x1b[0m\n").as_str());
            } else {
                for i in &self.answers {
                    if answers.contains(&i) {
                        output
                            .push_str(format!("\x1b[1;32m{}\x1b[0m\n", self.options[*i]).as_str());
                    } else {
                        output
                            .push_str(format!("\x1b[1;31m{}\x1b[0m\n", self.options[*i]).as_str());
                    }
                }
            }
        } else {
            output.push_str(format!("\x1b[1;32mCorrect!\x1b[0m\n").as_str());
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

        let mut questions: Vec<Question> = Vec::new();

        let lines: Vec<String> = file.lines().map(|s| s.to_string()).collect();
        let mut i: usize = 0;
        while i < lines.len() {
            let marker: &Option<char> = &lines[i].chars().nth(0);

            match marker {
                Some('?') => {
                    let question: String = lines[i][1..].to_string();
                    let mut question: Question = Question::new(question);

                    let mut j: usize = i + 1;
                    while j < lines.len() {
                        let marker: &Option<char> = &lines[j].chars().nth(0);
                        match marker {
                            Some('?') => {
                                questions.push(question);
                                break;
                            }
                            Some('-') => {
                                let _ =
                                    &question.add_option(lines[j][1..].trim().to_string(), false);
                                j += 1;
                                continue;
                            }
                            Some('+') => {
                                let _ =
                                    &question.add_option(lines[j][1..].trim().to_string(), true);
                                j += 1;
                                continue;
                            }
                            _ => {
                                j += 1;
                                continue;
                            }
                        }
                    }
                    i = j;
                    continue;
                }
                _ => {
                    i += 1;
                    continue;
                }
            }
        }

        return questions;
    }
}
