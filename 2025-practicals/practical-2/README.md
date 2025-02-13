# Telnet Database
Uses SQLite as the database to store people.

## Features
- Allows for concurrent connections (5 at most)
- allows for ADD, GET and DELETE commands
- keep alive and heartbeat allowing the server to periodically ping a client to detect a disconnect/ idle connection

