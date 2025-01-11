pub mod load_balancer {
    use std::collections::VecDeque;
    use std::net::IpAddr;

    use crate::request::Request;

    /// Node represents a replica in the distributed system.
    /// `address` is a url address for the replica
    /// `weight` is the weight dynamically calculated based on node performance.
    pub struct Node {
        pub address: IpAddr,
        pub weight: f32,
    }

    impl Node {
        /// Returns a new node based on the input parameters
        pub fn new(address: IpAddr, weight: f32) -> Self {
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
        buffer: VecDeque<Request>,
        nodes: Vec<Node>,
        available_nodes: u32,
        lamport_timestamp: u64,
    }
}
