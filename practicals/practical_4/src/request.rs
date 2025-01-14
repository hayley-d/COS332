pub mod http_request {
    use crate::error::my_errors::ErrorType;
    use colored::Colorize;
    use core::str;
    use log::error;
    use std::fmt::Display;

    #[derive(Debug)]
    pub struct Clock {
        lamport_timestamp: i64,
    }

    impl Clock {
        pub fn new() -> Self {
            Clock {
                lamport_timestamp: 0,
            }
        }
        pub fn increment_time(&mut self) -> i64 {
            let temp: i64 = self.lamport_timestamp;
            self.lamport_timestamp += 1;
            temp
        }
    }

    impl Default for Clock {
        fn default() -> Self {
            Self::new()
        }
    }

    #[derive(Debug)]
    pub enum Protocol {
        Http,
    }

    impl Display for Protocol {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Protocol::Http => write!(f, "HTTP/1.1"),
            }
        }
    }

    #[derive(Debug)]
    pub struct Header {
        pub title: String,
        pub value: String,
    }

    impl Display for Header {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} : {}", self.title, self.value)
        }
    }

    #[derive(Debug)]
    pub enum ContentType {
        Text,
        Html,
        Json,
    }

    impl Display for ContentType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ContentType::Text => write!(f, "text/plain"),
                ContentType::Html => write!(f, "text/html"),
                ContentType::Json => write!(f, "application/json"),
            }
        }
    }

    pub struct Request {
        pub request_id: i64,
        pub client_ip: String,
        pub headers: Vec<String>,
        pub body: String,
        pub method: HttpMethod,
        pub uri: String,
    }

    impl Display for Request {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Request: {{method: {}, path: {}, request_id: {},client_ip: {}}}",
                self.method, self.uri, self.request_id, self.client_ip
            )
        }
    }

    impl Request {
        pub fn print(&self) {
            println!("{} New Request:", ">>".red().bold());
            println!("{}{}", self.method.to_string().magenta(), self.uri.cyan());
        }

        pub fn new(
            buffer: &[u8],
            client_ip: String,
            request_id: i64,
        ) -> Result<Request, ErrorType> {
            // unwrap is safe as request has been parsed for any issues before this is called
            let request = String::from_utf8(buffer.to_vec()).unwrap();

            // split the request by line
            let request: Vec<&str> = request.lines().collect();

            if request.len() < 3 {
                error!(target: "error_logger","Recieved invalid request");
                return Err(ErrorType::ConnectionError(String::from("Invalid request")));
            }

            // get the http method from the first line
            let method: HttpMethod =
                HttpMethod::new(request[0].split_whitespace().collect::<Vec<&str>>()[0]);

            // get the uri from the first line
            let mut uri: String =
                request[0].split_whitespace().collect::<Vec<&str>>()[1].to_string();
            if uri == "/favicon.ico" {
                uri = "/".to_string();
            }

            // headers are the rest of the
            let mut headers: Vec<String> = Vec::with_capacity(request.len() - 1);
            let mut body: String = String::new();
            let mut flag = false;
            for line in &request[1..] {
                if line.is_empty() {
                    flag = true;
                    continue;
                }
                if flag {
                    body.push_str(line);
                } else {
                    headers.push(line.to_string());
                }
            }

            Ok(Request {
                request_id,
                client_ip,
                headers,
                body,
                method,
                uri,
            })
        }

        pub fn is_compression_supported(&self) -> bool {
            for header in &self.headers {
                let header = header.to_lowercase();

                if header.contains("firefox") {
                    return false;
                }

                if header.contains("accept-encoding") {
                    if header.contains(',') {
                        // multiple compression types
                        let mut encodings: Vec<&str> =
                            header.split(", ").map(|m| m.trim()).collect::<Vec<&str>>();
                        encodings[0] = encodings[0].split_whitespace().collect::<Vec<&str>>()[1];

                        for encoding in encodings {
                            if encoding == "gzip" || encoding.contains("gzip") {
                                return true;
                            }
                        }
                    } else if header
                        .to_lowercase()
                        .split_whitespace()
                        .collect::<Vec<&str>>()[1]
                        == "gzip"
                    {
                        return true;
                    }
                }
            }
            false
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum HttpCode {
        Ok,
        Created,
        BadRequest,
        Unauthorized,
        NotFound,
        MethodNotAllowed,
        RequestTimeout,
        Teapot,
        InternalServerError,
    }

    impl Display for HttpCode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                HttpCode::Ok => write!(f, "200 OK"),
                HttpCode::Created => write!(f, "201 Created"),
                HttpCode::BadRequest => write!(f, "400 Bad Request"),
                HttpCode::Unauthorized => write!(f, "401 Unauthorized"),
                HttpCode::NotFound => write!(f, "404 Not Found"),
                HttpCode::MethodNotAllowed => write!(f, "405 Method Not Allowed"),
                HttpCode::RequestTimeout => write!(f, "408 Request Timeout"),
                HttpCode::Teapot => write!(f, "418 I'm a teapot"),
                HttpCode::InternalServerError => write!(f, "500 Internal Server Error"),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum HttpMethod {
        GET,
        POST,
        PUT,
        PATCH,
        DELETE,
    }

    impl HttpMethod {
        pub fn new(method: &str) -> HttpMethod {
            if method.to_uppercase().contains("GET") {
                HttpMethod::GET
            } else if method.to_uppercase().contains("POST") {
                HttpMethod::POST
            } else if method.to_uppercase().contains("PUT") {
                HttpMethod::PUT
            } else if method.to_uppercase().contains("DELETE") {
                HttpMethod::DELETE
            } else {
                HttpMethod::PATCH
            }
        }
    }

    impl Display for HttpMethod {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                HttpMethod::GET => write!(f, "GET"),
                HttpMethod::POST => write!(f, "POST"),
                HttpMethod::PUT => write!(f, "PUT"),
                HttpMethod::PATCH => write!(f, "PATCH"),
                HttpMethod::DELETE => write!(f, "DELETE"),
            }
        }
    }
}
