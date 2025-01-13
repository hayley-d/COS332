pub mod load_balancer {
    use crate::rate_limiter_proto::rate_limiter_client::RateLimiterClient;
    use crate::rate_limiter_proto::RateLimitRequest;
    use crate::request::Request;
    use std::collections::{BTreeMap, VecDeque};
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::time::Duration;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpStream;
    use tokio::time::timeout;
    use tonic::transport::Channel;

    const RATELIMITERADDRESS: &str = "http://127.0.0.1:50051";

    /// Node represents a replica in the distributed system.
    /// `address` is a url address for the replica
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct Node {
        pub address: String,
    }

    impl Node {
        /// Returns a new node based on the input parameters
        pub fn new(address: String) -> Self {
            Node { address }
        }
    }

    pub struct LoadBalancer {
        pub buffer: VecDeque<Request>,
        pub nodes: Vec<Node>,
        pub lamport_timestamp: u64,
        pub ring: BTreeMap<u64, String>,
    }

    impl LoadBalancer {
        pub fn increment_time(&mut self) -> u64 {
            let temp = self.lamport_timestamp;
            self.lamport_timestamp += 1;
            temp
        }

        pub async fn insert(&mut self, request: Request) -> bool {
            let rate_limit_request = RateLimitRequest {
                ip_address: request.client_ip.clone(),
                endpoint: request.uri.clone(),
                request_id: request.request_id.to_string(),
            };

            // send request to rate limiter
            let mut client: RateLimiterClient<Channel> =
                match RateLimiterClient::connect(RATELIMITERADDRESS.to_string().clone()).await {
                    Ok(c) => c,
                    Err(_) => return false,
                };

            let response = match timeout(
                Duration::from_millis(10),
                client.check_request(rate_limit_request),
            )
            .await
            {
                Ok(Ok(value)) => value,
                Ok(Err(_)) => return false,
                Err(_) => {
                    return false;
                }
            };

            if response.into_inner().allowed {
                self.buffer.push_back(request);
                return true;
            }

            return false;
        }

        pub async fn new(addresses: &mut Vec<String>) -> Self {
            let mut ring = BTreeMap::new();

            // creates virtula nodes
            for node in addresses.clone() {
                let hash = Self::add_node(&node);
                ring.insert(hash, node.clone());

                /*for i in 0..available_nodes {
                    let virtual_node = format!("{}_{}", node, i);
                    let hash = Self::add_node(&virtual_node);
                    ring.insert(hash, node.clone());
                }*/
            }

            let mut nodes: Vec<Node> = Vec::new();
            for node in addresses {
                nodes.push(Node {
                    address: node.to_string(),
                });
            }

            LoadBalancer {
                buffer: VecDeque::new(),
                nodes,
                lamport_timestamp: 0,
                ring,
            }
        }

        // calculates the hash of the node address for the ring
        pub fn add_node<T: Hash>(address: &T) -> u64 {
            let mut hasher = DefaultHasher::new();
            address.hash(&mut hasher);
            hasher.finish()
        }

        // distribute requests based on consistent hash
        pub async fn distribute(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
            while let Some(request) = self.buffer.pop_front() {
                let node_address = match self.get_node(&request.client_ip) {
                    Some(address) => address.clone(),
                    _ => continue,
                };

                self.increment_time();

                let request = match serialize_request(request.request).await {
                    Ok(r) => r,
                    _ => continue,
                };

                let _ = send_request(request, node_address).await;
            }

            return Ok(());
        }

        /// Calculate the hash for a node using hasher instance
        pub fn get_node<H: Hash>(&self, node: &H) -> Option<&String> {
            let key = Self::add_node(node);

            self.ring
                .range(key..)
                .next()
                .map(|(_, node)| node)
                .or_else(|| self.ring.iter().next().map(|(_, node)| node))
        }
    }

    /// Sends a bytes array to the assigned node in the system
    async fn send_request(request: Vec<u8>, uri: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect(uri).await?;
        stream.write_all(&request).await?;
        Ok(())
    }

    /// Convert the http::Request struct into a byte array to send over the network
    async fn serialize_request(
        request: http::Request<Vec<u8>>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let (parts, body) = request.into_parts();

        let mut request_bytes: Vec<u8> = Vec::new();

        request_bytes.extend_from_slice(parts.method.as_str().as_bytes());
        request_bytes.extend_from_slice(b" ");
        request_bytes.extend_from_slice(
            parts
                .uri
                .path_and_query()
                .map_or(b"/".as_slice(), |pq| pq.as_str().as_bytes()),
        );
        request_bytes.extend_from_slice(b" ");
        request_bytes.extend_from_slice(format!("{:?}\r\n", parts.version).as_bytes());

        for (name, value) in &parts.headers {
            request_bytes.extend_from_slice(name.as_str().as_bytes());
            request_bytes.extend_from_slice(b": ");
            request_bytes.extend_from_slice(value.as_bytes());
            request_bytes.extend_from_slice(b"\r\n");
        }

        // Add the body
        request_bytes.extend_from_slice(&body);

        Ok(request_bytes)
    }
}
