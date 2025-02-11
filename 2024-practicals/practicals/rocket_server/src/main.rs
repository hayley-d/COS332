#[macro_use]
extern crate rocket;
use rand::Rng;
use rocket::http::{Cookie, CookieJar, Status};
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
    ids: Vec<String>,
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

// Struct for handling answer submissions
#[derive(Debug, FromForm)]
struct AnswerSubmission {
    question_id: String,
    answers: Vec<usize>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SubmitResponse {
    status: Status,
    message: String,
}

#[derive(Debug, rocket::FromForm)]
pub struct EmailRequest {
    #[field(validate=len(1..))]
    pub(crate) email: String,
}

#[launch]
async fn rocket() -> _ {
    let questions: HashMap<String, Question> = Question::parse_file().await;
    let mut ids: Vec<String> = Vec::new();

    for key in questions.keys() {
        ids.push(key.to_string());
    }

    let state: Arc<Mutex<AppState>> = Arc::new(Mutex::new(AppState {
        questions,
        user_scores: HashMap::new(),
        ids,
    }));

    rocket::build()
        .manage(state)
        .attach(Template::fairing())
        .mount("/", routes![submit_answer, show_results, email, home])
}

#[post("/results")]
async fn show_results() -> Template {
    // Render the email.html
    Template::render("email", context! {})
}

// Route: Server question home
#[get("/")]
async fn home(state: &rocket::State<Arc<Mutex<AppState>>>, cookies: &CookieJar<'_>) -> Template {
    let mut state = state.lock().await;
    let user_id = get_or_create_user(cookies);
    let (score, total) = state.user_scores.entry(user_id).or_insert((0, 0));
    println!("Current score is : {score}/{total}");
    let random_index = rand::thread_rng().gen_range(0..state.ids.len() - 1);
    let random_id = state.ids.get(random_index).unwrap();
    match state.questions.get(random_id) {
        Some(q) => {
            return Template::render(
                "index",
                context! {
                    question: q.question.clone(),
                    question_id: q.question_id.clone(),
                    option1: q.options[0].clone(),
                    option2: q.options[1].clone(),
                    option3: q.options[2].clone(),
                    option4: q.options[3].clone()
                },
            );
        }
        None => return Template::render("404", context! {}),
    };
}

#[post("/answer", data = "<form>")]
async fn submit_answer(
    form: rocket::form::Form<rocket::form::Contextual<'_, AnswerSubmission>>,
    state: &rocket::State<Arc<Mutex<AppState>>>,
    cookies: &CookieJar<'_>,
) -> Result<Template, Template> {
    if let Some(ref answer) = form.value {
        println!("Answer: {:?}", answer);

        let user_id = get_or_create_user(cookies);
        let mut correct: bool = false;
        let mut message: String = String::new();

        // get the state of the answer and the message associated
        match state.lock().await.questions.get(&answer.question_id) {
            Some(q) => {
                let answers = answer.answers.iter().map(|&a| a - 1).collect();
                correct = q.check_answer_correct(&answers);
                message = q.check_answer(answers.clone());
            }
            None => return Err(Template::render("400", context! {message: message})),
        };

        // Update the user score
        if let Some((score, total)) = state.lock().await.user_scores.get_mut(&user_id) {
            if correct {
                *score += 1;
            }
            *total += 1;
            return Ok(Template::render("answer", context! {message: message}));
        };
        return Err(Template::render("400", context! {}));
    }
    Err(Template::render("400", context! {}))
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

#[post("/email", data = "<form>")]
async fn email(
    form: rocket::form::Form<rocket::form::Contextual<'_, EmailRequest>>,
    state: &rocket::State<Arc<Mutex<AppState>>>,
    cookies: &CookieJar<'_>,
) -> Result<rocket::response::Flash<rocket::response::Redirect>, Template> {
    if let Some(ref email) = form.value {
        let user_id: String = get_or_create_user(cookies);
        let mut message = String::new();

        // Get Results and craft message
        if let Some((score, totoal)) = state.lock().await.user_scores.get(&user_id) {
            message.push_str(&format!(
                "You scored {}/{} on the Distributed Systems Test.\nWell Done! Play again soon ;)",
                score, totoal
            ));
        }
        // Clear User Results from HashMap
        state.lock().await.user_scores.remove(&user_id);

        // Email Results
        let mut attempts: usize = 1;
        while attempts > 0 {
            match rocket_server::mail::send_mail(message.clone(), email.email.clone()).await {
                Ok(_) => {
                    let message = rocket::response::Flash::success(
                        rocket::response::Redirect::to(uri!(home())),
                        "Email Sent",
                    );
                    return Ok(message);
                }
                Err(_) => {
                    attempts -= 1;
                }
            };
        }
    }
    return Err(Template::render("404", context! {message: ""}));
}
