extern crate ethereum_types;
extern crate merkle;

use ethereum_types::{H256, U256};
use merkle::*;
use std::collections::HashMap;

#[test]
fn create_tree() {
    let dummy_hex = "0x0101010101010101010101010101010101010101010101010101010101010101";
    let exp_root = "0x48ce19d92fe8d6b4be1d7744c1a798bde5d7f12ad192fe520aeae0462f3df29e";

    let leaves: HashMap<U256, H256> = [
        (U256::from(1), H256::from(dummy_hex)),
        (U256::from(2), H256::from(dummy_hex)),
        (U256::from(3), H256::from(dummy_hex)),
        (U256::from(4), H256::from(dummy_hex)),
    ]
    .iter()
    .cloned()
    .collect();

    let tree = Tree::build(leaves, DEFAULT_DEPTH).unwrap();
    assert_eq!(tree.root(), H256::from(exp_root));
}
