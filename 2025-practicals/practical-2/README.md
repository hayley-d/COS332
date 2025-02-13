# Telnet Database
This project implements a Telnet-based Friend Database Server using Rust. The server allows users to manage a list of friends and their phone numbers via a simple text-based interface over Telnet

## Commands
- `ADD <name> <phone>`: Adds a new friend to the database,
- `GET <name>`: Retrieves the phone number of the friend.
- `DELETE <name>`: Removes the friend from the database
- `EXIT`: Disconnects from the server.

## Features
1. **Keep-Alive & Heartbeat System**
    - The server sends a `PING` message every 10 seconds.
    - The client must respond with a `PONG` to stay connected.
    - If the client fails to respond within 15 seconds then it will automatically disconnect.

2. **Multi-User Capability**
    - The server supports multiple simultaneous Telnet connections.


## Implementation Details
- **Sockets:** Uses `libc` for raw socket handling (bind, listen, accept, read, write).
- **Database Handling:** A simple SQLite friend database managed with a `Mutex`.
- **Concurrency:** Uses `Tokio` for handling multiple clients asynchronously.
- **Timeout Handling:** The Keep-Alive system ensures idle connections are cleaned up.


