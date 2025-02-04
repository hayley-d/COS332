//! This module adds a deeper layer of complexity and security. Instead of checking the hash for
//! the entire file, it is split into blocks and a Merkle Tree is built to detect where the file
//! was modified.
pub mod merkle {
    use sha2::{Digest, Sha256};

    fn hash_block(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Splits the file into chunks (4kb blocks) and hashes each block.
    /// The tree is built using pairs of hashes until the root hash.
    pub fn build_merkel_tree(blocks: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        let mut current_level = blocks;

        while current_level.len() > 1 {
            let mut next_level = vec![];

            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    let mut combined = chunk.get(0).unwrap().clone();
                    combined.extend(chunk.get(1).unwrap());
                    next_level.push(hash_block(&combined));
                } else {
                    next_level.push(chunk.get(0).unwrap().clone());
                }
            }
            current_level = next_level;
        }

        current_level
    }
}
