pub mod my_errors {
    use std::fmt;

    #[derive(PartialEq, Eq)]
    pub enum ErrorType {
        SocketError(String),
        ReadError(String),
        WriteError(String),
        BadRequest(String),
        NotFound(String),
        InternalServerError(String),
        ProtocolError(String),
        ConnectionError(String),
    }

    impl std::error::Error for ErrorType {}

    impl ErrorType {
        pub fn get_msg(&self) -> &str {
            match self {
                ErrorType::SocketError(msg) => msg,
                ErrorType::ReadError(msg) => msg,
                ErrorType::WriteError(msg) => msg,
                ErrorType::BadRequest(msg) => msg,
                ErrorType::NotFound(msg) => msg,
                ErrorType::InternalServerError(msg) => msg,
                ErrorType::ProtocolError(msg) => msg,
                ErrorType::ConnectionError(msg) => msg,
            }
        }
    }

    impl fmt::Display for ErrorType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ErrorType::SocketError(msg) => write!(f, "Error with socket: {}", msg),
                ErrorType::ReadError(msg) => write!(f, "Error reading file: {}", msg),
                ErrorType::WriteError(msg) => write!(f, "Error writing to file: {}", msg),
                ErrorType::BadRequest(msg) => write!(f, "Error bad request: {}", msg),
                ErrorType::NotFound(msg) => write!(f, "Error resource not found: {}", msg),
                ErrorType::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
                ErrorType::ProtocolError(msg) => write!(f, "Protocol Error: {}", msg),
                ErrorType::ConnectionError(msg) => write!(f, "Connection Error: {}", msg),
            }
        }
    }

    impl fmt::Debug for ErrorType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ErrorType::SocketError(msg) => {
                    write!(
                        f,
                        "Socket Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::ReadError(msg) => {
                    write!(
                        f,
                        "Read Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::WriteError(msg) => {
                    write!(
                        f,
                        "Write Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::BadRequest(msg) => {
                    write!(
                        f,
                        "Bad Request Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }

                ErrorType::NotFound(msg) => {
                    write!(
                        f,
                        "Resource Not Found Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::InternalServerError(msg) => {
                    write!(
                        f,
                        "Internal Server Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::ProtocolError(msg) => {
                    write!(
                        f,
                        "Protocol Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::ConnectionError(msg) => write!(
                    f,
                    "Connection Error: {{ file: {}, line: {} message: {} }}",
                    file!(),
                    line!(),
                    msg
                ),
            }
        }
    }
}
