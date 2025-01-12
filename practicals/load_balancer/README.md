## Consistent Hashing Load Balancer

This reverse proxy server runs on port 3000.
This reverse proxy expects a .env file containing the node urls
```
NODE1=http://127.0.0.1:7878
NODE2=http://127.0.0.1:7879
```

This reverse proxy expects the following: 
- server runs on prost 3000 so port 3000 must be available
- 2 nodes running on ports 7878 and 7879
- rate limiter running on port 50051     
const RATELIMITERADDRESS: &str = "http://127.0.0.1:7879";

