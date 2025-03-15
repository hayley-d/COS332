use crate::ErrorType;
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
    pub(crate) request_id: i64,
    pub(crate) client_ip: String,
    pub(crate) headers: Vec<String>,
    pub(crate) body: Vec<u8>,
    pub(crate) method: HttpMethod,
    pub(crate) uri: String,
    // The image blob data if present
    pub(crate) image: Option<Vec<u8>>,
    // The name if present
    pub(crate) name: Option<String>,
    // The number if present
    pub(crate) number: Option<String>,
}

impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Request: {{method: {}, path: {}, request_id: {},client_ip: {},headers: {:?}}}",
            self.method, self.uri, self.request_id, self.client_ip, self.headers
        )
    }
}

impl Request {
    pub fn print(&self) {
        println!("{} New Request:", ">>".red().bold());
        println!("{}{}", self.method.to_string().magenta(), self.uri.cyan());
    }

    pub fn new(buffer: &[u8], client_ip: String, request_id: i64) -> Result<Request, ErrorType> {
        if let Some(body_start) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            // Split the buffer into two parts
            let header_part: Vec<u8> = buffer[..body_start].to_vec();
            let body: Vec<u8> = buffer[body_start + 4..].to_vec();

            let request = String::from_utf8_lossy(&header_part);
            let request: Vec<&str> = request.lines().collect();

            if request.len() < 3 {
                error!(target: "error_logger","Recieved invalid request");
                return Err(ErrorType::ConnectionError(String::from("Invalid request")));
            }

            // Get the method from the reqest line
            let method: HttpMethod =
                HttpMethod::new(request[0].split_whitespace().collect::<Vec<&str>>()[0]);

            // Get the URI from the request line
            let mut uri: String =
                request[0].split_whitespace().collect::<Vec<&str>>()[1].to_string();
            if uri == "/favicon.ico" {
                uri = "/".to_string();
            }

            // Parse the headers
            let headers: Vec<String> = request[1..]
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<String>>()[..]
                .to_vec();

            if let Some(boundary) = Request::extract_boundary(&headers) {
                println!("Extracted boundary: {}", boundary);
                let (image, name, number) = match Request::parse_multipart_form(&body, &boundary) {
                    Ok((i, name, num)) => (i, name, num),
                    Err(_) => {
                        log::error!(target:"error_logger","Failed to parse form data");
                        return Err(ErrorType::BadRequest(
                            "Failed to parse form data".to_string(),
                        ));
                    }
                };
                Ok(Request {
                    request_id,
                    client_ip,
                    headers,
                    body,
                    method,
                    uri,
                    image,
                    name,
                    number,
                })
            } else {
                log::info!(target:"request_logger","Request does not contain multipart form data");
                Ok(Request {
                    request_id,
                    client_ip,
                    headers,
                    body,
                    method,
                    uri,
                    image: None,
                    name: None,
                    number: None,
                })
            }
        } else {
            Err(ErrorType::ConnectionError(
                "Error splitting request".to_string(),
            ))
        }
    }

    // Extract the boundry from the Content-Type header
    fn extract_boundary(headers: &[String]) -> Option<String> {
        for header in headers {
            if header
                .to_lowercase()
                .starts_with("content-type: multipart/form-data;")
            {
                return header
                    .split("boundary=")
                    .nth(1)
                    .map(|b| b.trim().to_string());
            }
        }
        None
    }

    pub fn parse_multipart_form(
        body: &[u8],
        boundary: &str,
    ) -> Result<(Option<Vec<u8>>, Option<String>, Option<String>), ErrorType> {
        let boundary = format!("--{}", boundary);
        let parts: Vec<&[u8]> = body
            .split(|b| {
                body.windows(boundary.len())
                    .any(|w| w == boundary.as_bytes())
            })
            .collect();

        let mut image: Option<Vec<u8>> = None;

        let mut fields: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for part in parts {
            if part.is_empty() || part.starts_with(b"--") {
                continue;
            }
            let part_str = String::from_utf8_lossy(part);
            let mut lines = part_str.lines();

            if let Some(content_disposition) = lines.next() {
                if content_disposition.contains("form-data") {
                    let name = match Request::extract_field_name(content_disposition) {
                        Ok(name) => name,
                        _ => {
                            log::error!(target:"error_logger","Failed to extract field name");
                            return Err(ErrorType::BadRequest(
                                "Failed to extract field name".to_string(),
                            ));
                        }
                    };

                    if content_disposition.contains("filename=") {
                        let file_content_start = part_str
                            .find("\r\n\r\n")
                            .map(|i| i + 4)
                            .unwrap_or(part.len());

                        image = Some(part[file_content_start..].to_vec());
                    } else {
                        let value = lines.collect::<Vec<&str>>().join("\n");
                        fields.insert(name, value);
                    }
                }
            }
        }

        let name: String = match fields.get("name") {
            Some(name) => name.to_string(),
            None => {
                log::error!(target:"error_logger","Request missing name field");
                return Err(ErrorType::BadRequest(
                    "Request missing name field".to_string(),
                ));
            }
        };
        let number: String = match fields.get("number") {
            Some(num) => num.to_string(),
            None => {
                log::error!(target:"error_logger","Request missing number field");
                return Err(ErrorType::BadRequest(
                    "Request missing number field".to_string(),
                ));
            }
        };

        Ok((image, Some(name), Some(number)))
    }

    fn extract_field_name(header: &str) -> Result<String, String> {
        header
            .split(";")
            .find(|s| s.trim().starts_with("name="))
            .map(|s| {
                s.trim()
                    .trim_start_matches("name=")
                    .trim_matches('"')
                    .to_string()
            })
            .ok_or("Missing field name".to_string())
    }

    fn extract_filename(header: &str) -> Result<String, String> {
        header
            .split(";")
            .find(|s| s.trim().starts_with("filename="))
            .map(|s| {
                s.trim()
                    .trim_start_matches("filename=")
                    .trim_matches('"')
                    .to_string()
            })
            .ok_or("Missing filename".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_multipart_form_with_text_fields() {
        let boundary = "----WebKitFormBoundary123456";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\nAlice\r\n\
            --{boundary}\r\nContent-Disposition: form-data; name=\"number\"\r\n\r\n1234567890\r\n\
            --{boundary}--\r\n"
        );

        let (image, name, number) =
            crate::Request::parse_multipart_form(body.as_bytes(), boundary).unwrap();

        assert_eq!(name, Some(&"Alice".to_string()).cloned());
        assert!(name.is_some());
        assert_eq!(number, Some(&"1234567890".to_string()).cloned());
        assert!(number.is_some());
        assert!(image.is_none());
    }

    #[test]
    fn test_parse_multipart_form_with_file_upload() {
        let boundary = "----WebKitFormBoundary123456";
        let file_content = "Hello, this is a test file.";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\nAlice\r\n\
            --{boundary}\r\nContent-Disposition: form-data; name=\"number\"\r\n\r\n0674152597\r\n\
            --{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\nContent-Type: text/plain\r\n\r\n{file_content}\r\n\
            --{boundary}--\r\n"
        );

        let (image, name, number) =
            crate::Request::parse_multipart_form(body.as_bytes(), boundary).unwrap();

        assert_eq!(name, Some(&"Alice".to_string()).cloned());
        assert_eq!(number, Some(&"0674152597".to_string()).cloned());
        assert!(image.is_some());
        assert_eq!(String::from_utf8_lossy(&image.unwrap()), file_content);
    }

    #[test]
    fn test_parse_multipart_form_with_missing_fields() {
        let boundary = "----WebKitFormBoundary123456";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\nAlice\r\n\
            --{boundary}\r\nContent-Disposition: form-data; name=\"number\"\r\n\r\n12345\r\n\
            --{boundary}--\r\n"
        );

        let (image, name, number) =
            crate::Request::parse_multipart_form(body.as_bytes(), boundary).unwrap();

        assert_eq!(name, Some(&"Alice".to_string()).cloned());
        assert!(number.is_none());
        assert!(image.is_none());
    }

    #[test]
    fn test_parse_multipart_form_with_empty_body() {
        let boundary = "----WebKitFormBoundary123456";
        let body = "";

        let result = crate::Request::parse_multipart_form(body.as_bytes(), boundary);

        assert!(result.is_err());
    }
}
