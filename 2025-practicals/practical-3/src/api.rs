/// This module provides the core functionality for handling HTTP requests and
/// generating appropriate responses in an asynchronous server.
///
/// It includes:
/// - Utility functions for reading files.
/// - Handlers for various HTTP methods (GET, POST, PUT, PATCH, DELETE).
/// - Integration with shared state for caching and user management.
use crate::response::{MyDefault, Response};
use crate::server::SharedState;
use crate::{ContentType, HttpCode, HttpMethod, Request};
use colored::Colorize;
use log::{error, info};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Reads the content of a file located at `path` and returns it as a `Vec<u8>`.
///
/// # Arguments
/// - `path`: A string slice that holds the file path.
///
/// # Returns
/// A vector of bytes representing the file content.
///
/// # Panics
/// Panics if the file cannot be opened or read.
pub async fn read_file_to_bytes(path: &str) -> Vec<u8> {
    let metadata = fs::metadata(path).await.unwrap();
    let mut file = File::open(path).await.unwrap();
    let mut buffer: Vec<u8> = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer).await.unwrap();
    buffer
}

/// Fetches the byte content of a file from the cache or reads and caches it if not present.
///
/// # Arguments
/// - `state`: A shared, thread-safe state used for caching.
/// - `file_path`: The path to the file to read.
/// - `route_name`: The name of the route associated with the file.
///
/// # Returns
/// A vector of bytes representing the file content.
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

/// Handles incoming HTTP requests and routes them to the appropriate method-specific handler.
///
/// # Arguments
/// - `request`: The incoming HTTP request.
/// - `state`: A shared, thread-safe state used for managing server data and caching.
///
/// # Returns
/// A `Response` object generated based on the request.
pub async fn handle_response(request: Request, state: Arc<Mutex<SharedState>>) -> Response {
    match request.method {
        HttpMethod::GET => handle_get(request, state).await,
        HttpMethod::POST => handle_post(request, state).await,
        HttpMethod::PUT => handle_put(request).await,
        HttpMethod::PATCH => handle_patch(request).await,
        HttpMethod::DELETE => handle_delete(request, state).await,
    }
}

/// Handles HTTP GET requests, serving static files and handling special routes.
///
/// # Arguments
/// - `request`: The incoming GET request.
/// - `state`: A shared, thread-safe state used for managing server data and caching.
///
/// # Returns
/// A `Response` object with the appropriate content and status code.
async fn handle_get(request: Request, state: Arc<Mutex<SharedState>>) -> Response {
    let mut response = Response::default()
        .await
        .compression(request.is_compression_supported());

    match request.uri.as_str() {
        "/" => {
            info!(target: "request_logger","GET / from status 200");
            response.add_body(get_bytes(state, PathBuf::from(r"static/index.html"), "/").await);
        }
        "/home" => {
            info!(target: "request_logger","GET /home status: 200");
            response.add_body(get_bytes(state, PathBuf::from(r"static/home.html"), "/home").await);
        }
        "/coffee" => {
            info!(target: "request_logger","GET /coffee status: 418");
            return response
                .code(HttpCode::Teapot)
                .body(get_bytes(state, PathBuf::from(r"static/teapot.html"), "/coffee").await);
        }
        "/calculate" => {}
        _ => {
            error!(target: "error_logger","Failed to serve request GET {}", request.uri);
            info!(target: "request_logger","GET {} status: 404", request.uri);
            return response
                .code(HttpCode::BadRequest)
                .content_type(ContentType::Text)
                .body(get_bytes(state, PathBuf::from(r"static/404.html"), "/404").await);
        }
    }

    response
}

/// Handles HTTP POST requests for specific routes like `/signup` and `/login`.
///
/// # Arguments
/// - `request`: The incoming POST request.
/// - `state`: A shared, thread-safe state used for managing server data and caching.
///
/// # Returns
/// A `Response` object with the appropriate content and status code.
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
    response
        .body(String::from("Invalid post URI.").into())
        .code(HttpCode::BadRequest)
}

/// Handles HTTP PUT requests which are currently unsupported and return a `405` Method Not
/// Allowed.
///
/// # Arguments
/// - `request`: The incoming POST request.
///
/// # Returns
/// A `Response` object with the appropriate content and status code.
async fn handle_put(request: Request) -> Response {
    info!(target: "request_logger","PUT {} status 405", request.uri);

    Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::MethodNotAllowed)
}

/// Handles HTTP PATCH requests which are currently unsupported and return a `405` Method Not
/// Allowed.
///
/// # Arguments
/// - `request`: The incoming POST request.
///
/// # Returns
/// A `Response` object with the appropriate content and status code.
async fn handle_patch(request: Request) -> Response {
    info!(target: "request_logger","PATCH {} status 405", request.uri);
    Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::MethodNotAllowed)
}

/// Handles HTTP DELETE requests which are currently unsupported and return a `405` Method Not
/// Allowed.
///
/// # Arguments
/// - `request`: The incoming POST request.
///
/// # Returns
/// A `Response` object with the appropriate content and status code.

async fn handle_delete(request: Request, state: Arc<Mutex<SharedState>>) -> Response {
    info!(target: "request_logger","Delete {} status 405", request.uri);
    Response::default()
        .await
        .compression(request.is_compression_supported())
        .body(read_file_to_bytes("static/index.html").await)
        .code(HttpCode::MethodNotAllowed)
}

#[cfg(test)]
mod tests {

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
