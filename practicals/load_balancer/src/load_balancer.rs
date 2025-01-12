pub mod load_balancer {
    use crate::rate_limiter_proto::rate_limiter_client::RateLimiterClient;
    use crate::rate_limiter_proto::RateLimitRequest;
    use crate::request::Request;
    use hyper::client::conn::http1::Builder;
    use hyper_util::rt::TokioIo;
    use rand::Rng;
    use std::collections::VecDeque;
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio::time::timeout;
    use tonic::transport::Channel;

    const RATELIMITERADDRESS: &str = "http://127.0.0.1:7879";

    /// Node represents a replica in the distributed system.
    /// `address` is a url address for the replica
    /// `weight` is the weight dynamically calculated based on node performance.
    pub struct Node {
        pub address: String,
        pub weight: f32,
    }

    impl Node {
        /// Returns a new node based on the input parameters
        pub fn new(address: String, weight: f32) -> Self {
            Node { address, weight }
        }
    }

    impl Clone for Node {
        fn clone(&self) -> Self {
            Node {
                address: self.address.clone(),
                weight: self.weight,
            }
        }
    }

    impl Eq for Node {}

    impl PartialEq for Node {
        fn eq(&self, other: &Self) -> bool {
            self.weight == other.weight && self.address == other.address
        }
    }

    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(&other))
        }
    }

    impl Ord for Node {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            if self.weight < other.weight {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        }
    }

    pub struct LoadBalancer {
        pub buffer: VecDeque<Request>,
        pub nodes: Vec<Node>,
        pub available_nodes: u32,
        pub lamport_timestamp: u64,
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

        pub async fn new(
            mut available_nodes: u32,
            addresses: &mut Vec<String>,
        ) -> Result<Self, Box<dyn std::error::Error>> {
            let mut nodes: Vec<Node> = Vec::with_capacity(available_nodes as usize);
            let mut weights: Vec<f32> = Vec::with_capacity(available_nodes as usize);

            for (i, address) in addresses.clone().iter().enumerate() {
                let weight: f32 = match Self::get_weight(&address).await {
                    Ok(w) => w,
                    Err(_) => {
                        available_nodes -= 1;
                        addresses.remove(i);
                        continue;
                    }
                };
                weights.push(weight);
            }

            let total_weight: f32 = weights.iter().sum();

            let normalized_weights: Vec<f32> = weights
                .iter()
                .map(|&w| (w / total_weight) * 100.0)
                .collect();

            for (i, weight) in normalized_weights.iter().enumerate() {
                nodes.push(Node::new(addresses.get(i).unwrap().clone(), *weight));
            }

            nodes.sort_by(|a, b| b.cmp(&a));

            Ok(LoadBalancer {
                buffer: VecDeque::new(),
                nodes,
                available_nodes,
                lamport_timestamp: 0,
            })
        }
        pub async fn add_node(&mut self, address: String) {
            let weight = match Self::get_weight(&address).await {
                Ok(w) => w,
                Err(_) => {
                    return;
                }
            };

            let mut weights: Vec<f32> = Vec::with_capacity(self.available_nodes as usize + 1);

            let mut remove_nodes: Vec<usize> = Vec::new();

            for (i, node) in self.nodes.iter().enumerate() {
                let weight = match Self::get_weight(&node.address).await {
                    Ok(w) => w,
                    Err(_) => {
                        self.available_nodes -= 1;
                        remove_nodes.push(i);
                        continue;
                    }
                };
                weights.push(weight);
            }
            weights.push(weight);

            for i in remove_nodes {
                self.nodes.remove(i);
            }

            let total_weight = weights.iter().sum::<f32>();
            self.nodes.push(Node::new(address, weight));

            let normalized_weights: Vec<f32> = weights
                .iter()
                .map(|&w| (w / total_weight) * 100.0)
                .collect();

            for (i, weight) in normalized_weights.iter().enumerate() {
                if i < self.nodes.len() {
                    self.nodes[i].weight = *weight;
                }
            }
        }

        /// Calculcates the weight of a nodes as a percentage out of 100
        async fn get_weight(_: &str) -> Result<f32, Box<dyn std::error::Error>> {
            Ok(rand::thread_rng().gen_range(0..100) as f32)
        }

        async fn update_weighting(&mut self) {
            let mut weights: Vec<f32> = Vec::with_capacity(self.available_nodes as usize + 1);

            let mut remove_nodes: Vec<usize> = Vec::new();
            for (i, node) in self.nodes.iter().enumerate() {
                let weight = match Self::get_weight(&node.address).await {
                    Ok(w) => w,
                    Err(_) => {
                        self.available_nodes -= 1;
                        remove_nodes.push(i);
                        continue;
                    }
                };

                weights.push(weight);
            }

            for i in remove_nodes {
                self.nodes.remove(i);
            }

            let total_weight = weights.iter().sum::<f32>();

            let normalized_weights: Vec<f32> = weights
                .iter()
                .map(|&w| (w / total_weight) * 100.0)
                .collect();

            for (i, weight) in normalized_weights.iter().enumerate() {
                if self.nodes.get(i).is_some() {
                    self.nodes[i].weight = *weight;
                }
            }
        }

        pub async fn distribute(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
            let number_requests: usize = self.nodes.len();

            for node in self.nodes.clone() {
                let my_requests: i32 = (number_requests as f32 * node.weight).floor() as i32;

                for _ in 0..my_requests {
                    let time = self.increment_time();
                    if time % 100 == 0 {
                        self.update_weighting().await;
                    }

                    let request: Request = match self.buffer.pop_front() {
                        Some(r) => r,
                        _ => {
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Buffer empty",
                            )));
                        }
                    };

                    // establish connection and send request
                    let stream = TcpStream::connect(node.address.clone()).await.unwrap();
                    let io = TokioIo::new(stream);

                    let (mut sender, conn) = Builder::new()
                        .preserve_header_case(true)
                        .title_case_headers(true)
                        .handshake(io)
                        .await?;
                    tokio::task::spawn(async move {
                        if let Err(_) = conn.await {
                            println!("connection failed");
                        }
                    });

                    let resp = sender.send_request(request.request).await?;
                }
            }
            return Ok(());
        }
    }
}
