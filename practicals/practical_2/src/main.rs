use practical_2::question::Question;

#[tokio::main]
async fn main() {
    let questions: Vec<Question> = Question::parse_file().await;

    for q in questions {
        println!("{}", q.print());
    }
}
