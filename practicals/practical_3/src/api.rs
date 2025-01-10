use crate::response::{MyDefault, Response};
use crate::server::SharedState;
use crate::{ContentType, ErrorType, HttpCode, HttpMethod, Request};
use argon2::password_hash::SaltString;
use argon2::PasswordHash;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use colored::Colorize;
use log::{error, info};
use rand::rngs::OsRng;
use rand::Rng;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use uuid::Uuid;

pub async fn read_file_to_bytes(path: &str) -> Vec<u8> {
    let metadata = fs::metadata(path).await.unwrap();
    let mut file = File::open(path).await.unwrap();
    let mut buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer).await.unwrap();
    return buffer;
}

async fn get_bytes(
    state: Arc<Mutex<SharedState>>,
    file_path: PathBuf,
    route_name: &str,
) -> Vec<u8> {
    return match state.lock().await.get_cached_content(route_name).await {
        Some(b) => b,
        None => {
            state
                .lock()
                .await
                .read_and_cache_page(&file_path, route_name)
                .await
        }
    };
}

/// Entry point into the REST API
pub async fn handle_response(request: Request, state: Arc<Mutex<SharedState>>) -> Response {
    match request.method {
        HttpMethod::GET => handle_get(request, state).await,
        HttpMethod::POST => handle_post(request, state).await,
        HttpMethod::PUT => handle_put(request, state).await,
        HttpMethod::PATCH => handle_patch(request, state).await,
        HttpMethod::DELETE => handle_delete(request, state).await,
    }
}

async fn handle_get(request: Request, state: Arc<Mutex<SharedState>>) -> Response {
    let mut response = Response::default()
        .await
        .compression(request.is_compression_supported());
    if request.uri == "/" {
        info!(target: "request_logger","GET / from status 200");
        response.add_body(get_bytes(state, PathBuf::from(r"static/index.html"), "/").await);
    } else if request.uri == "/home" {
        info!(target: "request_logger","GET /home status: 200");
        response.add_body(get_bytes(state, PathBuf::from(r"static/home.html"), "/home").await);
    } else if request.uri == "/coffee" {
        info!(target: "request_logger","GET /coffee status: 418");
        return response
            .code(HttpCode::Teapot)
            .body(get_bytes(state, PathBuf::from(r"static/teapot.html"), "/coffee").await);
    } else {
        error!(target: "error_logger","Failed to serve request GET {}", request.uri);
        info!(target: "request_logger","GET {} status: 404", request.uri);
        return response
            .code(HttpCode::BadRequest)
            .content_type(ContentType::Text)
            .body(get_bytes(state, PathBuf::from(r"static/404.html"), "/404").await);
    }
    return response;
}

async fn handle_post(request: Request, state: Arc<Mutex<SharedState>>) -> Response {
    let mut response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .content_type(ContentType::Text);

    if request.uri == "/signup" {
        // parse the JSON into a hashmap
        info!("POST /signup from");
        let user: HashMap<String, String> = match serde_json::from_str(&request.body) {
            Ok(u) => u,
            Err(_) => {
                error!("Failed to parse JSON in request from");
                println!(
                    "{} {} {} {}",
                    ">>".red().bold(),
                    "Invalid JSON for".red(),
                    request.method.to_string().magenta(),
                    request.uri.cyan()
                );
                return response
                    .body(String::from("Invalid JSON.").into())
                    .code(HttpCode::BadRequest);
            }
        };

        // insert the new user into the file
        let session_id: Uuid = match state
            .lock()
            .await
            .add_user(user["username"].clone(), user["password"].clone())
            .await
        {
            Ok(s) => s,
            Err(_) => {
                error!(target: "error_logger","Failed to insert user into the database");

                println!(
                    "{} {} {} {}",
                    ">>".red().bold(),
                    "Error while creating new user ".red(),
                    request.method.to_string().magenta(),
                    request.uri.cyan()
                );

                return response
                    .body(String::from("Problem occured when attempting to add new user.").into())
                    .code(HttpCode::InternalServerError);
            }
        };

        response.add_header(
            String::from("Set-Cookie"),
            format!("session={}; HttpOnly", session_id),
        );

        return response
            .body(String::from("New user successfully created!").into())
            .code(HttpCode::Ok);
    } else if request.uri == "/login" {
        info!("POST /login from ");
        let user: HashMap<String, String> = match serde_json::from_str(&request.body) {
            Ok(u) => u,
            Err(_) => {
                error!("Failed to parse JSON");
                info!("POST /login status 404");
                println!(
                    "{} {} {} {}",
                    ">>".red().bold(),
                    "Invaid JSON for".red(),
                    request.method.to_string().magenta(),
                    request.uri.cyan()
                );
                return response
                    .body(String::from("Invalid JSON.").into())
                    .code(HttpCode::BadRequest)
                    .content_type(ContentType::Text);
            }
        };

        let input_username: &str = &user["username"];

        let session_id: Uuid = match state
            .lock()
            .await
            .find_user(input_username.to_string())
            .await
        {
            Ok(s) => s,
            Err(_) => {
                error!(target:"error_logger","Failed to find the user");
                println!(
                    "{} {} {} {}",
                    ">>".red().bold(),
                    "User not found for".red(),
                    request.method.to_string().magenta(),
                    request.uri.cyan()
                );
                return response
                    .body(String::from("No user exists with the provided details.").into())
                    .code(HttpCode::BadRequest)
                    .content_type(ContentType::Text);
            }
        };

        response.add_header(
            String::from("Set-Cookie"),
            format!("session={}; HttpOnly", session_id),
        );

        return response
            .body(String::from("Authentification successful!").into())
            .code(HttpCode::Ok);
    }
    error!("Failed to parse invalid POST request");
    return response
        .body(String::from("Invalid post URI.").into())
        .code(HttpCode::BadRequest);
}

async fn handle_put(request: Request, state: Arc<Mutex<SharedState>>) -> Response {
    info!("{}", request);

    let response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::MethodNotAllowed);

    return response;
}

async fn handle_patch(request: Request, state: Arc<Mutex<SharedState>>) -> Response {
    info!("PATCH {} status 404", request.uri);
    let response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::MethodNotAllowed);

    return response;
}

async fn handle_delete(request: Request, state: Arc<Mutex<SharedState>>) -> Response {
    let response = Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::BadRequest)
        .content_type(ContentType::Text);

    let file: HashMap<String, String> = match serde_json::from_str(&request.body) {
        Ok(u) => u,
        Err(_) => {
            error!("Failed to parse invalid JSON");
            return response
                .body(String::from("Invalid JSON").into())
                .code(HttpCode::BadRequest);
        }
    };

    let file_name: &String = &file["file_name"];

    let cookie_header: Vec<String> = request
        .headers
        .into_iter()
        .filter(|h| h.contains("Cookie: session="))
        .collect();

    let cookie_header = match cookie_header.get(0) {
        Some(h) => h,
        None => {
            error!("Attempt to delete without proper authentification from IP address");
            return response
                .body(String::from("Unable to delete file without proper authentification.").into())
                .code(HttpCode::BadRequest);
        }
    };

    let header_parts: Vec<&str> = cookie_header.split_whitespace().collect();

    let cookie_value: &str = match header_parts.get(1) {
        Some(v) => v,
        None => {
            error!("Attempt to delete without proper authentification from IP address");
            return response
                .body(String::from("Unable to delete file without proper authentification.").into())
                .code(HttpCode::BadRequest);
        }
    };

    // cookie_value = session=sessionID
    if verify_cookie(cookie_value).await {
        // session has been verified process the delete
        match fs::remove_file(file_name).await {
            Ok(_) => {
                return response
                    .body(String::from("File successfully deleted.").into())
                    .code(HttpCode::Ok);
            }
            Err(_) => {
                error!("Failed to delete file that does not exist");
                return response
                    .body(String::from("Unable to delete file: File does not exist.").into())
                    .code(HttpCode::BadRequest);
            }
        }
    }

    return response
        .body(String::from("Unable to delete file.").into())
        .code(HttpCode::BadRequest);
}

async fn insert_user(
    username: String,
    password: String,
    session: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let password = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = match argon2.hash_password(&password, salt.as_salt()) {
        Ok(hash) => hash,
        Err(_) => {
            error!(target: "error_logger","Failed to create new user");
            return Err(Box::new(ErrorType::InternalServerError(String::from(
                "Problem occured when creating password",
            ))));
        }
    };

    let mut file_input: Vec<u8> = username.into_bytes();
    file_input.push(b'|');
    file_input.extend_from_slice(hash.to_string().as_bytes());
    file_input.push(b'|');
    file_input.extend_from_slice(session.as_bytes());
    let mut file = OpenOptions::new()
        .append(true)
        .open("static/users.txt")
        .await
        .expect("cannot open file");

    match file.write(&file_input).await {
        Ok(_) => (),
        Err(_) => {
            error!(target:"error_logger","Failed to write to database");
            return Err(Box::new(ErrorType::InternalServerError(String::from(
                "Problem occured when writing user to db",
            ))));
        }
    };

    Ok(())
}

/// Validates a password against a hashed password.
///
/// Uses Argon2 to verify if the provided password matches the stored hash.
///
/// # Arguments
/// - `password`: The plaintext password provided by the user.
/// - `hashed_password`: The stored hashed password.
///
/// # Returns
/// - `Ok(true)` if the password matches the hash.
/// - `Ok(false)` if the password does not match the hash.
/// - `Err(ErrorType)` if a validation error occurs.
fn validate_password(password: &str, hashed_password: &str) -> Result<bool, ErrorType> {
    let argon2 = Argon2::default();

    let parsed_hash = PasswordHash::new(hashed_password).map_err(|_| {
        error!(target: "error_logger","Failed to validated hashed password");
        ErrorType::InternalServerError(String::from(
            "Problem occurred when validating the password",
        ))
    })?;

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => {
            error!(target: "error_logger","Login attempt failed due to incorrect password");
            return Err(ErrorType::BadRequest(String::from("Incorrect Password")));
        }
    }
}

/// Generates a random session ID.
///
/// # Returns
/// - A `String` representing the session ID.
fn generate_session_id() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect()
}

/// Verifies the session cookie.
///
/// Reads the `users.txt` file to check if the provided session cookie matches an active session.
///
/// # Arguments
/// - `cookie`: The session cookie to verify.
///
/// # Returns
/// - `true` if the session is valid.
/// - `false` otherwise.
async fn verify_cookie(cookie: &str) -> bool {
    if cookie.starts_with("session=") {
        return match fs::read_to_string("static/users.txt").await {
            Ok(f) => {
                let cookie_value: &str = cookie.split('=').collect::<Vec<&str>>()[1];
                f.contains(cookie_value)
            }
            Err(_) => false,
        };
    }
    false
}

#[cfg(test)]
mod tests {

    use crate::api::verify_cookie;

    #[tokio::test]
    async fn test_verify_cookie() {
        let cookie: String = String::from("session=sloth101");
        let res = verify_cookie(&cookie).await;
        assert_eq!(res, true);
    }

    /*#[tokio::test]
    async fn test_signup() {
        let request_body = json!({
            "username": "hayley",
            "password": "password"
        })
        .to_string();

        let request = Request {
            request_id: 0,
            client_ip:
            headers: Vec::new(),
            body: request_body,
            method: HttpMethod::POST,
            uri: "/signup".to_string(),
        };
        let logger: Arc<Mutex<Logger>> = Arc::new(Mutex::new(Logger::new("server.log")));
        let response: Response = handle_post(request).await;
        assert_eq!(response.code, HttpCode::Ok);
    }*/
    /*#[tokio::test]
    async fn test_login() {
        let request_body = json!({
            "username": "hayley",
            "password": "password"
        })
        .to_string();

        let request = Request {
            headers: Vec::new(),
            body: request_body,
            method: HttpMethod::POST,
            uri: "/login".to_string(),
        };
        let logger: Arc<Mutex<Logger>> = Arc::new(Mutex::new(Logger::new("server.log")));
        let response: Response = handle_post(request, logger).await;
        assert_eq!(response.code, HttpCode::Ok);
    }*/
}
