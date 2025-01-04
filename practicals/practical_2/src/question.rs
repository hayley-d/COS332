pub struct Question {
    question: String,
    options: Vec<String>,
    answers: Vec<usize>,
}

impl Question {
    pub fn new(questions: String, options: Vec<String>, answers: Vec<usize>) -> Self {
        return Question {
            question,
            options,
            answers,
        };
    }
}
