extern crate ethereum_types;
extern crate tiny_keccak;

use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::error::Error;
use std::ops::Sub;
use std::vec::Vec;

use ethereum_types::{H256, U256};
use std::fmt;
use tiny_keccak::keccak256;

pub const DEFAULT_DEPTH: usize = 257;

pub type Leaves = HashMap<U256, H256>;
pub type Tree = Vec<RefCell<Leaves>>;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    root: H256,
    depth: usize,
    default_nodes: Leaves,
    nodes: Tree,
}

pub type Result<T> = std::result::Result<T, MerkleError>;

#[derive(Debug, Clone)]
pub enum MerkleError {
    DepthErr,
}

impl Error for MerkleError {
    fn description(&self) -> &str {
        match *self {
            MerkleError::DepthErr => "very large depth",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MerkleError::DepthErr => None,
        }
    }
}

impl fmt::Display for MerkleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MerkleError::DepthErr => write!(f, "very large depth"),
        }
    }
}

impl MerkleTree {
    pub fn new(leaves: Leaves, depth: usize) -> MerkleTree {
        let len = leaves.len();
        let cap = 2.0_f64.powi(len as i32);

        if len > (cap.floor() as usize) {
            panic!("123");
        }

        let default_nodes = default_nodes(depth);
        let nodes = create(leaves, &default_nodes, depth);
        let root: H256;

        {
            let leave: &RefCell<Leaves> = nodes.get(nodes.len() - 1).unwrap();
            root = leave.borrow().get(&null_u256()).unwrap().clone();
        }

        MerkleTree {
            root,
            depth,
            default_nodes,
            nodes,
        }
    }

    pub fn root(&self) -> H256 {
        self.root.clone()
    }
}

fn create(leaves: Leaves, default_nodes: &Leaves, depth: usize) -> Vec<RefCell<Leaves>> {
    let mut tree = vec![RefCell::new(leaves)];

    {
        let mut _tree_level: RefCell<Leaves>;

        for level in 0..depth - 1 {
            {
                let level: &RefCell<Leaves> = tree.get(level).unwrap();
                _tree_level = level.clone();
            }

            let mut next_level: Leaves = HashMap::new();
            let mut prev_index = U256::from(0);

            let mut keys = Vec::new();

            {
                for (k, _) in _tree_level.borrow().iter() {
                    keys.push(k.clone())
                }
                keys.sort();
            }

            for index in &keys {
                {
                    let div = index / 2;

                    let mut value: Vec<u8> = _tree_level
                        .borrow()
                        .get(index)
                        .unwrap_or(&null_h256())
                        .to_vec();

                    let mut hash;

                    if index % 2 == null_u256() {
                        let mut value2: Vec<u8> =
                            default_nodes.get(&U256::from(level)).unwrap().to_vec();
                        value.append(&mut value2);
                        hash = keccak256(value.as_slice());
                    } else {
                        if index.clone() == prev_index + 1 {
                            let mut value2: Vec<u8> = _tree_level
                                .borrow_mut()
                                .get(&U256::from(prev_index))
                                .unwrap_or(&null_h256())
                                .to_vec();
                            value2.append(&mut value);
                            hash = keccak256(value2.as_slice());
                        } else {
                            let mut value2: Vec<u8> =
                                default_nodes.get(&U256::from(level)).unwrap().to_vec();
                            value2.append(&mut value);
                            hash = keccak256(value2.as_slice());
                        }
                    }

                    next_level.insert(U256::from(div), H256::from(hash));
                }
                prev_index = index.clone();
            }

            {
                let level = RefCell::new(next_level);
                _tree_level = level.clone();
                tree.push(level);
            }
        }
    }
    tree
}

fn default_nodes(depth: usize) -> Leaves {
    let mut nodes = HashMap::new();
    nodes.insert(null_u256(), null_h256());

    for level in 1..depth {
        let next_level = level.sub(1);
        let mut previous = nodes.get(&U256::from(next_level)).unwrap().to_vec();
        let mut previous2 = previous.clone();
        previous.append(&mut previous2);
        let hash = keccak256(&previous);
        nodes.insert(U256::from(level), H256::from(hash));
    }
    nodes
}

fn null_u256() -> U256 {
    U256::from(0)
}

fn null_h256() -> H256 {
    H256::from(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree3() {
        let mut leaves = HashMap::new();
        leaves.insert(
            U256::from(1),
            H256::from("0x0101010101010101010101010101010101010101010101010101010101010101"),
        );
        leaves.insert(
            U256::from(2),
            H256::from("0x0101010101010101010101010101010101010101010101010101010101010101"),
        );
        let tree = MerkleTree::new(leaves, DEFAULT_DEPTH);
    }

    #[test]
    fn create_default_nodes() {
        let exp = super::DEFAULT_DEPTH;
        let default_nodes = super::default_nodes(exp);

        assert_eq!(default_nodes.len(), exp);
    }
}
