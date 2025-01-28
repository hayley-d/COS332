# Practial 8
Core Requirements:
- Watch a single file for changes on the local system.
- Compare the local file to the one on the FTP server
- Implement FTP commands from scratch using sockets.
- If the file is altered, fetch the file from the FTP client and replace the loca version.
- Handle situations when the file is locked/editing.
- logs for demo purposes

### Sepcial Features
#### 1. File Event Notificaions (instead of polling)
- Use a crate called `notify` that hooks OS mechanisms to detect file changes.
- More efficient than polling.
- reduces overhead and scales better.

#### 2. Merkle Tree
- Verfiy parts of a large file:
- hash using SHA256

#### 3. Errors
- if file is locked queue the replacement until it is available.

## Features
### 1. File System Events
- Uses file system events instead of polling which improves responsiveness and reduces system load.
- Uses the `notify` crate for file system events functionality.
- responds to modification and deletion events.
- Immediate notification of changes rather than periodic polling.

### 2. Merkel Tree Implementation
- Used for efficient file change detection.
- splits the file into chuncks and creates a hash tree.
- allows for future expansion to partial file verification.
- more sophisitated than simple hash comparison.


