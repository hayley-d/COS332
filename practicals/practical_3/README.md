# Roadmap for HTTP/2 Server

## Socket and TLS Setup
* Improve Error handling and logging
* Add mechanism for graceful shutdown

## Http/2 Protocol Basics
* Preface Handiling (validate preface)
* Frame Parsing (encoding and decoding logic)

## Implement Flow Control
* Implement pre-stream and connection wide flow control
* Dynamically adjust window size

## Multiplexing Streams
* Handle multiple streams concurrently over the same connection
* Create stream registry to track active strams
* handle stream prioritization

## Server Features
* Parse headers to extreact request details
* Build minimal HTTP/2 response
* Implement basic REST API
* gRPC support

## Error Handling and Compliance
* Send appropriate RST_STREAM frames on errors
* Response with GOAWAY frames during connection termination or protocol violations
* Implement SETTING fram handilng to negotiate server capabilities

## Add Flare
* gRPC support
* Connection pooling: Handle simultaneous connections efficiently
* Metrics and logging: Include Prometheus metrics or structured logging
* Security: Implement DDoS protection (Rate limiting, IP blacklisting)
