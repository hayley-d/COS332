use std::io::ErrorKind;

use uuid::Uuid;

pub struct Request {
    pub request_id: Uuid,
    pub client_ip: String,
    pub uri: String,
    pub request: Vec<u8>,
}

impl Request {
    pub fn print(&self) {
        println!("{} New Request:", ">>");
        println!("{}", String::from_utf8(self.request.clone()).unwrap());
    }

    pub fn new(buffer: &[u8], client_ip: String) -> Result<Request, Box<dyn std::error::Error>> {
        let request = String::from_utf8(buffer.to_vec()).unwrap();

        // split the request by line
        let request: Vec<&str> = request.lines().collect();

        if request.len() < 3 {
            return Err(Box::new(std::io::Error::new(
                ErrorKind::Other,
                "request invalid length",
            )));
        }

        // get the uri from the first line
        let mut uri: String = request[0].split_whitespace().collect::<Vec<&str>>()[1].to_string();
        if uri == "/favicon.ico" {
            uri = "/".to_string();
        }

        return Ok(Request {
            request_id: Uuid::new_v4(),
            client_ip,
            uri,
            request: buffer.to_vec(),
        });
    }
}
