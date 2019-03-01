# Merkle tree

Merkle tree implementation.

## Example
```rust
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
        H256::from("0x0202020202020202020202020202020202020202020202020202020202020202"),
    );
    leaves.insert(
        U256::from(3),
        H256::from("0x0303030303030303030303030303030303030303030303030303030303030303"),
    );
    leaves.insert(
        U256::from(4),
        H256::from("0x0404040404040404040404040404040404040404040404040404040404040404"),
    );

    let tree = merkle::Tree::build(leaves, merkle::DEFAULT_DEPTH).unwrap();
    let proof = tree.create_proof(U256::from(1)).unwrap();
    let valid = tree.verify_proof(
        U256::from(1),
        H256::from("0x0101010101010101010101010101010101010101010101010101010101010101"),
        tree.root(),
        proof.as_slice(),
    );
    
    println!("root: {:?}", tree.root());
    println!("is valid member?: {:?}", valid);
}
```