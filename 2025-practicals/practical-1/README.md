## HTTP Server in Rust
This repository showcases an advanced Rust-based HTTP server with rich functionality, designed for high performance and secure communication. By integrating Redis and PostgreSQL, the server demonstrates real-world backend capabilities while incorporating robust networking principles.

### Features

#### Performace and Reliablilty
- **Concurrent Connections:** Efficiently manages multiple client connections asynchronously using Rust's `tokio` runtime.
- **Redis Caching:** Integrates Redis for caching frequently accessed documents, reducing database load.
- **PostgreSQL Integration:** Seamlessly connects to a PostgreSQL database for persistent data storage enabling client authentification.
- **Error Handling:** Utilizes `log4rs` for structured and detailed server-side logging.
- **Graceful Shutdown:** Ensures clean resource management during server termination.

#### Advanced HTTP Functionality
- **Gzip Compression:** Optimizes data transfer by compressing HTTP responses.
- **Persistent Connections:** Implements HTTP/1.1 keep-alive to reduce connection overhead.
- **Custom Headers:** Enables dynamic inclusion of custom headers for enhanced client-server communication.
- **Client Authentification:** Supports authentification through session cookies.
- **Support for JSON payloads:** Able to handle POST request with JSON payloads using the `serde` crate for serialization and deserialization.
- **Raw Socket Management:** Leverages `libc` for direct socket creation and control, showcasing low-level networking expertise.
- **HTTP status codes:** supports the following codes: `200`,`201`,`400`,`404`,`405`,`408`,`418` and `500`

#### Security and Authentification
- **TLS Support:** Ensures encrypted communication for sensitive data transmission.
- **Rate Limiting:** Integrates with a rate limiter to limit the amount of requests sent by IP address enabling DDoS protection.
- **Password Security:** Employs hashing for secure password storage and verification.

### Dependancies
Ensure that you have Redis and PostgreSQL intalled.
A Redis server should be running locally on port 6379.
A PostgreSQL server should be running locally on port 5432 and the .env file should contain the Database connection string in the following format.
```
DATABASE_URL=postgres://<username>:<password>@localhost:5432/<database_name>
```

The PostgreSQL database should contain the following table: 
```
CREATE TABLE users( 
    user_id SERIAL PRIMARY KEY, 
    username VARCHAR(50) UNIQUE NOT NULL, 
    password TEXT NOT NULL, 
    created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP, 
    session_id UUID DEFAULT NULL 
);
```


### Getting Started
#### Prerequisites
* Rust (version 1.8 or later)
* A running instance of Redis on port 6379
* A running instance of PostgreSQL with database setup running on port 5432

#### Setup Instructions
1. Install dependancies
```bash
cargo build
```
2. Start Redis (if not already)
```bash
redis-server
```
3. Start PostgreSQL (if not already)
```bash
sudo systemctl restart postgresql
```
4. Run the server (port is optional defaults to 7878)
```bash
cargo run <port>
```


