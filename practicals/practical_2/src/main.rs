use practical_2::question::Question;

#[tokio::main]
async fn main() {
    Question::parse_file().await;
}
