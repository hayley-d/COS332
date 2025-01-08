use std::io::Write;

use chrono::{DateTime, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::{read_file_to_bytes, ContentType, Header, HttpCode, Protocol};

#[derive(Debug)]
pub struct Response {
    pub protocol: Protocol,
    pub code: HttpCode,
    pub content_type: ContentType,
    pub body: Vec<u8>,
    pub compression: bool,
    pub headers: Vec<Header>,
}

#[allow(async_fn_in_trait)]
pub trait MyDefault {
    async fn default() -> Self;
}

impl MyDefault for Response {
    async fn default() -> Self {
        let mut response = Response::new(Protocol::Http, HttpCode::Ok, ContentType::Html, true);

        response.add_body(read_file_to_bytes("static/index.html").await);

        return response;
    }
}

impl Response {
    pub fn add_header(&mut self, title: String, value: String) {
        self.headers.push(Header { title, value });
    }

    pub fn to_bytes(&mut self) -> Vec<u8> {
        // Response line: HTTP/1.1 <status code>
        let response_line: String = format!("{} {}\r\n", self.protocol, self.code);

        let body: Vec<u8>;

        if !self.compression {
            body = self.body.clone();
        } else {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(&self.body)
                .expect("Failed to write body to gzip encoder");
            body = encoder.finish().expect("Failed to finish gzip compression");
            //self.add_header(String::from("Content-Encoding"), String::from("gzip"));
        }

        self.add_header(String::from("Content-Length"), body.len().to_string());

        let mut headers: Vec<String> = Vec::new();

        for header in &self.headers {
            headers.push(header.to_string());
        }

        println!("{:?}", headers);

        let mut response = Vec::new();
        response.extend_from_slice(response_line.as_bytes());
        response.extend_from_slice(headers.join("\r\n").as_bytes());
        response.extend_from_slice(b"\r\n\r\n");
        response.extend_from_slice(&body);

        return response;
    }

    pub fn add_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }

    pub fn new(
        protocol: Protocol,
        code: HttpCode,
        content_type: ContentType,
        compression: bool,
    ) -> Self {
        let body = Vec::with_capacity(0);

        // Date Header
        let now: DateTime<Utc> = Utc::now();
        let date = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let mut headers: Vec<Header> = vec![
            Header {
                title: String::from("Server"),
                value: String::from("Ferriscuit"),
            },
            Header {
                title: String::from("Date"),
                value: date,
            },
            Header {
                title: String::from("Cache-Control"),
                value: String::from("no-store"),
            },
            Header {
                title: String::from("Content-Type"),
                value: content_type.to_string(),
            },
        ];

        if compression {
            headers.push(Header {
                title: String::from("Content-Encoding"),
                value: String::from("gzip"),
            });
        }

        return Response {
            protocol,
            code,
            content_type,
            body,
            compression,
            headers,
        };
    }

    pub fn code(mut self, code: HttpCode) -> Self {
        self.code = code;
        return self;
    }

    pub fn content_type(mut self, content_type: ContentType) -> Self {
        self.content_type = content_type;
        return self;
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        return self;
    }

    pub fn compression(mut self, compression: bool) -> Self {
        self.compression = compression;
        // add header
        if compression {
            for header in &self.headers {
                if header.title == "Content-Encoding" {
                    return self;
                }
            }
            self.add_header(String::from("Content-Encoding"), String::from("gzip"));
        } else {
            let mut index: isize = -1;
            for (i, _) in self.headers.iter().enumerate() {
                if &self.headers[i].title == "Content-Encoding" {
                    index = i as isize;
                }
            }

            if index > 0 {
                self.headers.remove(index as usize);
            }
        }
        return self;
    }
}
