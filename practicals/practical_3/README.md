# Roadmap for HTTP/2 Server
CREATE TABLE users(
    user_id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    session_id UUID DEFAULT NULL
);

# Dependancies
- Install redis (sudo apt install redis/sudo dnf install redis)
- start the redis server(redis-server) (starts the server on 127.0.0.1:6379)
- stop the redis server with redis-cli shutdown
- test the redis instance wtih redis-cli ping
- PostgreSQL with the above table
- .env file with database connection string DATABASE_URL

## Additional Features
- gzip compression
- TLS
- Persistent Connections using the Connection header
- Custom caching
- Detailed logging
- Authentification (using session cookie)
- cache support (Redis) response caching

COMING SOON
- Upgrade header (websockets)
    Request
    ```
    {
        GET /chat HTTP/1.1
        Host: example.com
        Connection: Upgrade             // Tells the server that the client want to upgrade the connection to a different protocol
        Upgrade: websocket              // Indicates teh client wants to updgrade to WebSocket protocol
        Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
        Sec-WebSocket-Version: 13
    }
    ```

    Response
    ```
    {
        HTTP/1.1 101 Switching Protocols
        Upgrade: websocket
        Connection: Upgrade
        Sec-WebSocket-Accept: x3JJHMbDL1EzLkh9tHk7QfY9XUWeDYld6R2Gb3BGv7Y=
    }
    ```
- Security: Implement DDoS protection (Rate limiting, IP bla:cklisting)
