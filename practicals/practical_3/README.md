# Roadmap for HTTP/2 Server
# Dependancies
- Install redis (sudo apt install redis/sudo dnf install redis)
- start the redis server(redis-server) (starts the server on 127.0.0.1:6379)
- stop the redis server with redis-cli shutdown
- test the redis instance wtih redis-cli ping


## Additional Features
- gzip compression
- TLS
- Persistent Connections using the Connection header
- Custom caching
- Detailed logging

- Rate limiter (coming soon) and IP Blacklisting (with redis)
- Upgrade header (websockets)
- cache support (Redis) response caching
- Metrics and logging: Include Prometheus metrics or structured logging
- Security: Implement DDoS protection (Rate limiting, IP bla:cklisting)
