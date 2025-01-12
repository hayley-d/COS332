use uuid::Uuid;

pub struct Request {
    pub request_id: Uuid,
    pub client_ip: String,
    pub uri: String,
    pub request: hyper::Request<hyper::body::Incoming>,
}

impl Request {
    pub fn new(
        mut uri: String,
        client_ip: String,
        request: hyper::Request<hyper::body::Incoming>,
    ) -> Request {
        // get the uri from the first line
        if uri == "/favicon.ico" {
            uri = "/".to_string();
        }

        Request {
            request_id: Uuid::new_v4(),
            client_ip,
            uri,
            request,
        }
    }
}
