use std::time::SystemTime;

// Structure to track file versions and changes
#[derive(Clone)]
pub struct FileVersion {
    merkle_root: Vec<u8>,
    timestamp: SystemTime,
    version: u64,
    signature: Option<Vec<u8>>, // For HMAC verification
}

pub struct MerkleNode {
    hash: Vec<u8>,
    children: Option<(Box<MerkleNode>, Box<MerkleNode>)>,
    chunk_range: Option<(usize, usize)>, // Track chunk boundaries for partial updates
}
