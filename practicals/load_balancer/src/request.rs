use http::{Method, Uri, Version};
use uuid::Uuid;

pub struct Request {
    pub request_id: Uuid,
    pub client_ip: String,
    pub uri: String,
    pub request: http::Request<Vec<u8>>,
}

impl Request {
    pub fn new(mut uri: String, client_ip: String, mut request: http::Request<Vec<u8>>) -> Request {
        // get the uri from the first line
        if uri == "/favicon.ico" {
            uri = "/".to_string();
        }

        let request_id = Uuid::new_v4();

        // add the request ID to the headers
        request
            .headers_mut()
            .insert("X-Request-ID", request_id.to_string().parse().unwrap());

        Request {
            request_id,
            client_ip,
            uri,
            request,
        }
    }
}

pub fn buffer_to_request(buffer: Vec<u8>) -> Result<http::Request<Vec<u8>>, String> {
    // Find the position of the headers-body delimiter (\r\n\r\n)
    let delimiter = match buffer.windows(4).position(|window| window == b"\r\n\r\n") {
        Some(pos) => pos,
        None => return Err("Malformed request: missing headers-body delimiter".to_string()),
    };

    let (header_part, body) = buffer.split_at(delimiter + 4);

    let request_str: String = match std::str::from_utf8(header_part) {
        Ok(s) => s.to_string(),
        Err(_) => return Err("Invalid UTF-8 sequence in buffer".to_string()),
    };

    let mut lines = request_str.split("\r\n");

    let request_line: String = match lines.next() {
        Some(l) => l.to_string(),
        None => return Err("Missing request line".to_string()),
    };

    let mut request_line = request_line.split_whitespace();

    let method: Method = match request_line.next() {
        Some(m) => match m.parse::<Method>() {
            Ok(m) => m,
            _ => return Err("Invalid HTTP method".to_string()),
        },
        _ => return Err("Missing HTTP method".to_string()),
    };

    let uri: Uri = match request_line.next() {
        Some(u) => match u.parse::<Uri>() {
            Ok(u) => u,
            _ => return Err("Invalid URI".to_string()),
        },
        _ => return Err("Missing URI".to_string()),
    };

    let version: Version = match request_line.next() {
        Some(v) => match v {
            "HTTP/1.0" => Version::HTTP_10,
            "HTTP/1.1" => Version::HTTP_11,
            _ => return Err("Invalid HTTP Version".to_string()),
        },
        None => return Err("Missing HTTP version".to_string()),
    };

    let mut request_builder = http::Request::builder()
        .method(method)
        .uri(uri)
        .version(version);

    for line in lines.by_ref() {
        if line.is_empty() {
            break;
        }

        let mut header_parts = line.splitn(2, ": ");

        let name = match header_parts.next() {
            Some(n) => n,
            _ => return Err("Malformed Header".to_string()),
        };

        let value = match header_parts.next() {
            Some(n) => n,
            _ => return Err("Malformed Header".to_string()),
        };

        request_builder = request_builder.header(name, value);
    }

    let request = match request_builder.body(body.to_vec()) {
        Ok(r) => r,
        _ => return Err("Failed to build request".to_string()),
    };

    Ok(request)
}
