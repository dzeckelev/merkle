extern crate ethereum_types;
extern crate tiny_keccak;

use ethereum_types::{H256, U256};
use std::collections::hash_map::HashMap;
use tiny_keccak::keccak256;

fn test() {
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

    let tree = merkle::MerkleTree::build(leaves, merkle::DEFAULT_DEPTH).unwrap();
    println!("{:?}", tree.root());
    let proof = tree.create_proof(U256::from(2)).unwrap();
    println!("{:?}", proof.len());
    println!("proof hash: {:?}", keccak256(proof.as_slice()));

    let valid = merkle::verify_proof(
        U256::from(2),
        H256::from("0x0101010101010101010101010101010101010101010101010101010101010101"),
        tree.root(),
        proof.as_slice(),
    );
    println!("{:?}", valid);
}

fn main() {
    test();
}
