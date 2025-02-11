use std::collections::HashMap;
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
        Question {
            question_id: Uuid::new_v4(),
            question,
            options: Vec::new(),
            answers: Vec::new(),
        }
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
        output
    }

    pub fn check_answer(&self, answers: Vec<usize>) -> String {
        let mut output: String = String::new();
        if answers != self.answers {
            output.push_str("Incorrect the question answers are:\n");

            if self.answers.is_empty() {
                output.push_str("No correct answers\n");
            } else {
                for i in &self.answers {
                    output.push_str(format!("{}\n", self.options[*i]).as_str());
                }
            }
        } else {
            output.push_str("Correct!\n");
        }
        output
    }

    pub fn check_answer_correct(&self, answers: &Vec<usize>) -> bool {
        if *answers != self.answers {
            false
        } else {
            true
        }
    }

    pub async fn parse_file() -> HashMap<Uuid, Question> {
        let file = match fs::read_to_string("file.txt").await {
            Ok(f) => f,
            Err(_) => {
                eprintln!("Error reading file");
                std::process::exit(1);
            }
        };

        let mut questions: HashMap<Uuid, Question> = HashMap::new();

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
                                questions.insert(question.question_id, question);

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

        questions
    }

    pub fn generate_html_page(&self, client_id: String) -> Vec<u8> {
        let options = self
            .options
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, opt)| {
                acc.push_str(&format!(
                    "<div><input type='checkbox' name='answer' value='{}' /> {} </div>",
                    i + 1,
                    opt
                ));
                acc
            });

        format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Distributed Sysytems Test</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css" rel="stylesheet">
</head>
<body>
<div class="container mt-5">
    <h1>{}</h1>
    <form id="question-form" class="mt-4">
        <input type="hidden" name="uuid" id="uuid" value="{}" />
        <input type="hidden" name="client_id" id="client_id" value="{}" />

        {}
        <button type="button" class="btn btn-primary mt-3" onclick="submitAnswer()">Submit Answer</button>
        <button type="button" class="btn btn-primary mt-3"><a href="/">New question</a></button>

    </form>
    <div id="response" class="mt-4"></div>
</div>
<script>
    function submitAnswer() {{
        // Get the UUID
        const uuid = document.getElementById('uuid').value;
        const client_id = document.getElementById('client_id').value;


        // Get all checked checkboxes
        const checkedAnswers = Array.from(
            document.querySelectorAll('input[name="answer"]:checked')
        ).map(input => Number(input.value));

        // Create the JSON payload
        const payload = {{
            uuid: uuid,
            client_id: client_id,
            answers: checkedAnswers,
        }};

        // Send the JSON payload via fetch
        fetch('/answer', {{
            method: 'POST',
            headers: {{
                'Content-Type': 'application/json',
            }},
            body: JSON.stringify(payload),
        }})
        .then(response => response.text())
        .then(data => {{
            document.getElementById('response').innerHTML = `
                <div class="alert alert-info">${{data}}</div>`;
        }})
        .catch(err => {{
            document.getElementById('response').innerHTML = `
                <div class="alert alert-danger">Error: ${{err}}</div>`;
        }});
    }}
</script>
</body>
</html>
"#,
            self.question, self.question_id,client_id, options
        ).as_bytes().to_vec()
    }
}
