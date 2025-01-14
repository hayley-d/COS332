## HTTP Server in Rust
This repository showcases an advanced Rust-based HTTP server designed as a question-answer game. The client is prompted with a question and options and the client has to select the correct answer. 
### Features

#### Performace and Reliablilty
- **Concurrent Connections:** Efficiently manages multiple client connections asynchronously using Rust's `tokio` runtime.
- **Error Handling:** Utilizes `log4rs` for structured and detailed server-side logging.
- **Graceful Shutdown:** Ensures clean resource management during server termination.

#### Advanced HTTP Functionality
- **Gzip Compression:** Optimizes data transfer by compressing HTTP responses.
- **Persistent Connections:** Implements HTTP/1.1 keep-alive to reduce connection overhead.
- **Custom Headers:** Enables dynamic inclusion of custom headers for enhanced client-server communication.
- **Support for JSON payloads:** Able to handle POST request with JSON payloads using the `serde` crate for serialization and deserialization.
- **Raw Socket Management:** Leverages `libc` for direct socket creation and control, showcasing low-level networking expertise.
- **HTTP status codes:** supports the following codes: `200`,`201`,`400`,`404`,`405`,`408`,`418` and `500`

#### Security and Authentification
- **TLS Support:** Ensures encrypted communication for sensitive data transmission.

### Getting Started
#### Prerequisites
* Rust (version 1.8 or later)

#### Setup Instructions
1. Install dependancies
```bash
cargo build
```
4. Run the server (port is optional, server defaults to 7878)
```bash
cargo run <port>
```


