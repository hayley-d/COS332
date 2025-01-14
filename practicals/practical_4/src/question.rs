use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Eq)]
pub struct Question {
    question_id: Uuid,
    question: String,
    options: Vec<String>,
    answers: Vec<usize>,
}

impl PartialEq for Question {
    fn eq(&self, other: &Self) -> bool {
        self.question_id == other.question_id
    }
}

impl Question {
    pub fn new(question: String) -> Self {
        return Question {
            question_id: Uuid::new_v4(),
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
        let mut output = format!("Question: {}\n", self.question);
        for (i, option) in self.options.iter().enumerate() {
            let formatted_option = format!("({}) {}\n", i + 1, option);
            output.push_str(formatted_option.as_str());
        }
        return output;
    }

    pub fn check_answer(&self, answers: Vec<usize>) -> String {
        let mut output: String = String::new();
        if answers != self.answers {
            output.push_str(format!("Incorrect the question answers are:\n").as_str());

            if self.answers.is_empty() {
                output.push_str(format!("No correct answers\n").as_str());
            } else {
                for i in &self.answers {
                    if answers.contains(&i) {
                        output.push_str(format!("{}\n", self.options[*i]).as_str());
                    } else {
                        output.push_str(format!("{}\n", self.options[*i]).as_str());
                    }
                }
            }
        } else {
            output.push_str(format!("Correct!\n").as_str());
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
