#[macro_use]
extern crate rocket;

use rand::Rng;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::response::content::RawHtml;
use rocket::serde::json::Json;
use rocket::{Build, Request, Response, Rocket};
use rocket_dyn_templates::{context, Template};
use rocket_server::question::Question;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AppState {
    questions: HashMap<String, Question>,
    user_scores: HashMap<String, (usize, usize)>,
    ids: Vec<Uuid>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            questions: HashMap::new(),
            user_scores: HashMap::new(),
            ids: Vec::new(),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r AppState {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        todo!()
    }
}

// Struct for handling answer submissions
#[derive(Debug, serde::Serialize, serde::Deserialize, FromForm)]
struct AnswerSubmission {
    question_id: String,
    client_id: String,
    answers: Vec<usize>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SubmitResponse {
    status: Status,
    message: String,
}

// Route: Submit an answer and validate the solutions
#[post("/submit", data = "form")]
async fn submit_answer(
    answer: rocket::form::Form<AnswerSubmission>,
    state: &'a State<Arc<Mutex<AppState>>>,
    cookies: &'a CookieJar,
) -> Template {
    let answer = answer.into_inner();

    let user_id = get_or_create_user(cookies);
    let mut correct: bool = false;
    let mut message: String = String::new();

    let state = state.lock().await;

    // get the state of the answer and the message associated
    match state.questions.get(&answer.question_id) {
        Some(q) => {
            correct = q.check_answer_correct(&answer.answers);
            message = q.check_answer(answer.answers);
        }
        None => {
            return Err(Status::BadRequest);
        }
    };

    // Update the user score
    if let Some((score, total)) = state.user_scores.get_mut(&user_id) {
        if correct {
            *score += 1;
        }

        *total += 1;
        Template::render("answer", context! {message: message})
    };
    Template::render("400", context! {message: message})
}

#[derive(Debug, serde::Serialize, serde::Deserialize, FromForm)]
struct EmailRequest {
    #[field(validate=len(3..))]
    pub(crate) email: String,
}

// Recieves the email over POST request and sends the test resutls
#[post("/email", data = "form")]
async fn email(
    email: rocket::form::Form<EmailRequest>,
    state: &rocket::State<Arc<Mutex<AppState>>>,
    cookies: &CookieJar<'_>,
) -> Template {
    let email = email.into_inner();
    let state = state.lock().await;
    let user_id: String = get_or_create_user(cookies);
    let mut message = String::new();

    // Get Results and craft message
    if let Some((score, totoal)) = state.lock().await.user_scores.get(user_id) {
        message.push_str(&format!(
            "You scored {}/{} on the Distributed Systems Test./nWell Done! Play again soon ;)",
            score, totoal
        ));
    }

    // Email Results
    // send_mail(message,email.email);

    // Clear User Results from HashMap
    state.lock().await.user_scores.remove(user_id);

    Template::render("index", context! {message: message})
}

#[launch]
fn rocket() -> _ {
    let state: Arc<Mutex<AppState>> = Arc::new(Mutex::new(AppState::new()));

    rocket::build()
        .manage(state)
        .attach(Template::fairing())
        .mount(
            "/",
            routes![get_question, submit_answer, show_results, email, home],
        )
}

// Render the email results page
#[get("/results")]
async fn show_results() -> RawHtml<String> {
    // Render the email.html
    Template::render("email", context! {message: message})
}

// Route: Server question home
#[get("/")]
async fn home(state: &rocket::State<Arc<Mutex<AppState>>>, cookies: &CookieJar<'_>) -> Template {
    let state = state.lock().await;
    let user_id = get_or_create_user(cookies);
    let (score, total) = state.user_scores.entry(user_id).or_insert((0, 0));
    let random_index = rand::thread_rng().gen_range(0..state.ids.len() - 1);
    let random_id = state.ids.get(random_index).unwrap();
    let random_question = state.questions.get(random_id).unwrap();
    Template::render("index", context! {question: random_question})
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
