pub mod ldap_structures {
    use std::collections::HashMap;
    use std::sync::Arc;

    use tokio::sync::Mutex;

    /// Nodes for an object in the DIT.
    ///
    /// `dn`: Distinguished Name is used to uniquely identify the node.
    /// `attrubutes`: Describe the DNS record type with the corresponding values.
    /// `children`: Child nodes.
    #[derive(Debug)]
    pub struct Node {
        dn: String,
        attributes: HashMap<Attribute, String>,
        children: HashMap<String, Arc<Mutex<Node>>>,
    }

    impl Node {
        pub fn new(dn: String) -> Self {
            Node {
                dn,
                attributes: HashMap::new(),
                children: HashMap::new(),
            }
        }

        /// Adds a child to the node.
        ///
        /// # Argument
        /// `child`: A child node wrapped in an Arc<Mutex<_>>.
        pub async fn add_child(&mut self, child: Arc<Mutex<Node>>) {
            self.children
                .insert(child.lock().await.dn.clone(), child.clone());
        }

        /// Adds an attribute to the current node.
        ///
        /// # Arguments
        /// `(attribute,value)`: The Attribute enum and corresponding value.
        pub fn set_attribute(&mut self, (attribute, value): (Attribute, String)) {
            self.attributes.insert(attribute, value);
        }

        pub async fn query(&self, dn: &str) -> Option<String> {
            if self.dn == dn {
                let mut result = format!("DN: {}", self.dn);
                for (key, value) in &self.attributes {
                    result.push_str(&format!("\n{}: {}", key, value));
                }
                return Some(result);
            } else {
                for child in self.children.values() {
                    let child = child.lock().await;
                    if let Some(result) = Box::pin(child.query(dn)).await {
                        return Some(result);
                    }
                }
            }
            None
        }
    }

    /// DNS Record Types
    /// `A`: A record type for IPv4 address
    /// `AAAA`: AAAA record type for the IPv6 address.
    /// `MX`: MX record type for mail server specification. (mail exchange)
    /// `NS`: NS record type for authoritative name server (name server)
    /// `CNAME`: Map hostname to another hostname
    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    pub enum Attribute {
        A,
        AAAA,
        MX,
        NS,
        CNAME,
    }

    impl std::fmt::Display for Attribute {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Attribute::A => {
                    write!(f, "A Record")
                }
                Attribute::AAAA => {
                    write!(f, "AAAA Record")
                }
                Attribute::MX => {
                    write!(f, "MX Record")
                }
                Attribute::NS => {
                    write!(f, "NS Record")
                }
                Attribute::CNAME => {
                    write!(f, "CNAME Record")
                }
            }
        }
    }
}
