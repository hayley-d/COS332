#[macro_use]
extern crate rocket;

use rand::Rng;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::response::content::RawHtml;
use rocket::serde::json::Json;
use rocket::{Build, Request, Rocket, State};
use rocket_server::question::Question;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AppState {
    questions: HashMap<String, Question>,
    user_scores: HashMap<String, (usize, usize)>,
    ids: Vec<Uuid>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r AppState {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.guard::<&State<AppState>>().await {
            Outcome::Success(state) => Outcome::Success(state.inner()),
            Outcome::Forward(_) => Outcome::Forward(()),
        }
    }
}

// Struct for handling answer submissions
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AnswerSubmission {
    question_id: Uuid,
    selected_index: usize,
}

// Route: Serve the next question as HTML
#[get("/question")]
async fn get_question(state: &State<AppState>, cookies: &CookieJar<'_>) -> RawHtml<String> {
    let user_id = get_or_create_user(cookies);

    let mut user_scores = state.user_scores;

    let random_index = rand::thread_rng().gen_range(0..state.ids.len() - 1);
    let random_id = state.ids.get(random_index).unwrap();

    let random_question = state.questions.get(random_id).unwrap();

    let (score, total) = user_scores.entry(user_id).or_insert((0, 0));

    if *total < state.questions.len() {
        let question = random_question;
        *total += 1;

        let options_html: String = question
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                format!(
                    r#"<input type="radio" name="selected_index" value="{}"> {}<br>"#,
                    i, option
                )
            })
            .collect();

        RawHtml(format!(
            r#"
            <!DOCTYPE html>
            <html>
            <body>
                <h1>Question</h1>
                <p>{}</p>
                <form action="/submit" method="post">
                    {}
                    <button type="submit">Submit Answer</button>
                </form>
                <form action="/question" method="get">
                    <button type="submit">Next Question</button>
                </form>
                <form action="/results" method="get">
                    <button type="submit">Exit</button>
                </form>
            </body>
            </html>
            "#,
            question.question, options_html
        ))
    } else {
        RawHtml(
            r#"
            <!DOCTYPE html>
            <html>
            <body>
                <h1>End of Quiz</h1>
                <p>There are no more questions. Click "Exit" to see your results.</p>
                <form action="/results" method="get">
                    <button type="submit">Exit</button>
                </form>
            </body>
            </html>
        "#
            .to_string(),
        )
    }
}

// Route: Submit an answer
#[post("/submit", format = "json", data = "<answer>")]
fn submit_answer(
    answer: Json<AnswerSubmission>,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
) -> Status {
    let user_id = get_or_create_user(cookies);

    // Validate the answer
    let mut user_scores = state.user_scores;

    if let Some((score, total)) = user_scores.get_mut(&user_id) {
        let question_index = *total - 1; // Index of the last served question
        if question_index < state.questions.len() {
            let question = &state.questions[&question_index];

            if question.correct_index == answer.selected_index {
                *score += 1;
            }
            return Status::Ok;
        }
    }

    Status::BadRequest
}

// Generate or retrieve user ID from cookies
fn get_or_create_user(cookies: &CookieJar<'_>) -> String {
    if let Some(cookie) = cookies.get("user_id") {
        Uuid::parse_str(cookie.value())
            .unwrap_or_else(|_| create_new_user(cookies))
            .to_string()
    } else {
        create_new_user(cookies).to_string()
    }
}

// Create a new user ID and set it as a cookie
fn create_new_user(cookies: &CookieJar<'_>) -> Uuid {
    let user_id = Uuid::new_v4();
    cookies.add(Cookie::new("user_id", user_id.to_string()));
    user_id
}

#[get("/results")]
async fn show_results(state: &State<AppState>, cookies: &CookieJar<'_>) -> RawHtml<String> {
    let user_id: String = get_or_create_user(cookies);

    if let Some((score, total)) = state.user_scores.get(&user_id) {
        RawHtml(format!(
            r#"
            <!DOCTYPE html>
            <html>
            <body>
                <h1>Results</h1>
                <p>Your score: {}/{}</p>
                <p>Thank you for taking the quiz!</p>
            </body>
            </html>
            "#,
            score, total
        ))
    } else {
        RawHtml(
            r#"
            <!DOCTYPE html>
            <html>
            <body>
                <h1>Error</h1>
                <p>Could not find your results. Please try again.</p>
            </body>
            </html>
        "#
            .to_string(),
        )
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(AppState {
            questions: HashMap::new(),
            user_scores: HashMap::new(),
        })
        .mount("/", routes![get_question, submit_answer, show_results])
}
