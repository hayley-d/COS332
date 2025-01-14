/// This module provides the core functionality for handling HTTP requests and
/// generating appropriate responses in an asynchronous server.
///
/// It includes:
/// - Utility functions for reading files.
/// - Handlers for various HTTP methods (GET, POST, PUT, PATCH, DELETE).
/// - Integration with shared state for caching and user management.

pub mod question_api {
    use log::{error, info};
    use tokio::fs::{self, File};
    use tokio::io::AsyncReadExt;

    use crate::request::http_request::{ContentType, HttpCode, HttpMethod, Request};
    use crate::response::http_response::{MyDefault, Response};

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

    /// Handles incoming HTTP requests and routes them to the appropriate method-specific handler.
    ///
    /// # Arguments
    /// - `request`: The incoming HTTP request.
    /// - `state`: A shared, thread-safe state used for managing server data and caching.
    ///
    /// # Returns
    /// A `Response` object generated based on the request.
    pub async fn handle_response(request: Request) -> Response {
        match request.method {
            HttpMethod::GET => handle_get(request).await,
            HttpMethod::POST => handle_post(request).await,
            HttpMethod::PUT => handle_put(request).await,
            HttpMethod::PATCH => handle_patch(request).await,
            HttpMethod::DELETE => handle_delete(request).await,
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
    async fn handle_get(request: Request) -> Response {
        let mut response = Response::default()
            .await
            .compression(request.is_compression_supported());
        if request.uri == "/" {
            info!(target: "request_logger","GET / from status 200");
            response.add_body(read_file_to_bytes("static/index.html").await);
        } else if request.uri == "/question" {
            info!(target: "request_logger","GET /home status: 200");
            // IMPLEMENT
            todo!()
        } else {
            error!(target: "error_logger","Failed to serve request GET {}", request.uri);
            info!(target: "request_logger","GET {} status: 404", request.uri);
            return response
                .code(HttpCode::BadRequest)
                .content_type(ContentType::Text)
                .body(read_file_to_bytes("static/404.html").await);
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
    async fn handle_post(request: Request) -> Response {
        let mut response = Response::default()
            .await
            .compression(request.is_compression_supported())
            .body(read_file_to_bytes("static/index.html").await)
            .content_type(ContentType::Text);

        if request.uri == "/signup" {
            // parse the JSON into a hashmap
            info!("POST /signup from");

            return response
                .body(String::from("New user successfully created!").into())
                .code(HttpCode::Ok);
        } else if request.uri == "/login" {
            info!("POST /login from ");
            return response
                .body(String::from("Invalid JSON.").into())
                .code(HttpCode::BadRequest)
                .content_type(ContentType::Text);
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
    async fn handle_delete(request: Request) -> Response {
        info!(target: "request_logger","Delete {} status 405", request.uri);
        Response::default()
            .await
            .compression(request.is_compression_supported())
            .body(read_file_to_bytes("static/index.html").await)
            .code(HttpCode::MethodNotAllowed)
    }
}
