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

pub fn create(leaves: Leaves, default_nodes: Leaves, depth: usize) -> Vec<RefCell<Leaves>> {
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

pub fn new_default_nodes(depth: usize) -> Leaves {
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

    //    fn new_tree() -> Tree {
    //        let tree = Tree::new();
    //        tree.add_branch(HashMap::new());
    //        tree
    //    }

    #[test]
    fn new_tree2() {
        let mut leaves = HashMap::new();
        let mut default_nodes = HashMap::new();

        let tree = create(leaves, default_nodes, DEFAULT_DEPTH);
    }

    //    #[test]
    //    fn add_branch() {
    //        let tree = new_tree();
    //
    //        assert_eq!(tree.len(), 1);
    //    }
    //
    //    #[test]
    //    fn leaf() {
    //        let test_branch = 0;
    //        let test_key = U256::from("1232132133453535435345345345");
    //        let exp = H256::from([1_u8; 32]);
    //        let tree = new_tree();
    //        tree.add_leaf(test_branch, test_key, exp);
    //        let res = tree.leaf(test_branch, test_key);
    //
    //        assert_eq!(res, exp);
    //    }
    //
    #[test]
    fn create_default_nodes() {
        let exp = super::DEFAULT_DEPTH;
        let default_nodes = super::new_default_nodes(exp);

        assert_eq!(default_nodes.len(), exp);
    }
    //
    //    #[test]
    //    fn create() {
    //        let default_nodes = super::new_default_nodes(2);
    //        println!("{:?}", default_nodes);
    //
    //
    //        let mut nodes = HashMap::new();
    //        nodes.insert(U256::from(1), H256::from("0x0101010101010101010101010101010101010101010101010101010101010101"));
    //        nodes.insert(U256::from(2), H256::from("0x0101010101010101010101010101010101010101010101010101010101010101"));
    //
    //       let tree =  super::create(nodes, default_nodes, 2);
    //        println!("{:?}", tree);
    //    }
    //
    //    #[test]
    //    fn sort_keys() {
    //        let tree = new_tree();
    //        tree.add_branch(HashMap::new());
    //        let v1 = H256::from([0_u8;32]);
    //        tree.add_leaf(tree.len() - 1, U256::from(5), v1);
    //    }
}
