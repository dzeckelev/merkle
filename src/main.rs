extern crate ethereum_types;

use ethereum_types::{H256, U256};
use std::collections::hash_map::HashMap;

fn main() {
    let mut leaves = HashMap::new();
    leaves.insert(
        U256::from(1),
        H256::from("0x0101010101010101010101010101010101010101010101010101010101010101"),
    );
    leaves.insert(
        U256::from(2),
        H256::from("0x0101010101010101010101010101010101010101010101010101010101010101"),
    );
    leaves.insert(
        U256::from(3),
        H256::from("0x0101010101010101010101010101010101010101010101010101010101010101"),
    );
    leaves.insert(
        U256::from(4),
        H256::from("0x0101010101010101010101010101010101010101010101010101010101010101"),
    );

    let tree = merkle::MerkleTree::new(leaves, merkle::DEFAULT_DEPTH);
    println!("{:?}", tree.root());
}
