extern crate ethereum_types;
extern crate tiny_keccak;

use std::collections::hash_map::HashMap;
use std::error::Error;
use std::ops::Sub;
use std::rc::Rc;
use std::sync::RwLock;
use std::vec::Vec;

use ethereum_types::{H256, U256};
use std::fmt;
use tiny_keccak::keccak256;

pub const DEFAULT_DEPTH: usize = 257;

pub type Nodes = HashMap<U256, H256>;
pub type Levels = Vec<Nodes>;

pub type Result<T> = std::result::Result<T, MerkleError>;

#[derive(Debug)]
pub struct MerkleTree {
    root: H256,
    depth: usize,
    default_nodes: Nodes,
    tree: RwLock<Rc<Levels>>,
}

#[derive(Debug, Clone)]
pub enum MerkleError {
    ErrSmallDepth,
}

impl Error for MerkleError {
    fn description(&self) -> &str {
        match *self {
            MerkleError::ErrSmallDepth => "small depth",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MerkleError::ErrSmallDepth => None,
        }
    }
}

impl fmt::Display for MerkleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MerkleError::ErrSmallDepth => write!(f, "small depth"),
        }
    }
}

impl MerkleTree {
    pub fn new(nodes: Nodes, depth: usize) -> MerkleTree {
        let len = nodes.len();
        let cap = 2.0_f64.powi(depth as i32);

        if (len as f64) > cap {
            // TODO
            panic!("123");
        }

        let default_nodes = default_nodes(depth);
        let levels = create_tree(nodes, &default_nodes, depth);
        let root = root(&levels);

        MerkleTree {
            root,
            depth,
            default_nodes,
            tree: RwLock::new(Rc::new(levels)),
        }
    }

    pub fn root(&self) -> H256 {
        self.root.clone()
    }
}

fn root(levels: &Levels) -> H256 {
    let level: &Nodes = levels.get(&levels.len() - 1).unwrap();
    level.get(&null_u256()).unwrap().clone()
}

fn sort_keys(level: &Nodes) -> Vec<U256> {
    let mut keys = Vec::new();
    for (k, _) in level.iter() {
        keys.push(k.clone())
    }
    keys.sort();
    keys
}

fn calc_hash(left: &mut Vec<u8>, right: &mut Vec<u8>) -> [u8; 32] {
    left.append(right);
    keccak256(left.as_slice())
}

#[allow(dead_code)]
fn create_hash(
    level: usize,
    prev_index: &U256,
    node_index: &U256,
    default_nodes: &Nodes,
    tree_level: &Nodes,
) -> [u8; 32] {
    let get_value = |nodes: &Nodes, index: &U256| -> Vec<u8> {
        nodes.get(index).unwrap_or(&null_h256()).to_vec()
    };

    let mut left;
    let mut right;

    if node_index % 2 == null_u256() {
        left = get_value(tree_level, node_index);;
        right = get_value(default_nodes, &U256::from(level));
    } else {
        if node_index.clone() == prev_index + 1 {
            left = get_value(tree_level, prev_index);
            right = get_value(tree_level, node_index);;
        } else {
            left = get_value(default_nodes, &U256::from(level));
            right = get_value(tree_level, node_index);;
        }
    }
    calc_hash(&mut left, &mut right)
}

fn fill_tree(tree: &mut Levels, default_nodes: &Nodes, depth: usize) {
    for level in 0..depth - 1 {
        let mut next_level: Nodes = HashMap::new();
        let mut prev_index = null_u256();
        let mut keys = sort_keys(tree.get(level).unwrap());

        for node_index in keys {
            {
                let tree_level: &Nodes = tree.get(level).unwrap();
                let div = node_index / 2;
                let mut hash =
                    create_hash(level, &prev_index, &node_index, default_nodes, tree_level);

                next_level.insert(U256::from(div), H256::from(hash));
            }
            prev_index = node_index.clone();
        }

        tree.push(next_level);
    }
}

fn create_tree(nodes: Nodes, default_nodes: &Nodes, depth: usize) -> Levels {
    let mut levels = vec![nodes];
    fill_tree(&mut levels, &default_nodes, depth);
    levels
}

fn default_nodes(depth: usize) -> Nodes {
    let mut nodes = HashMap::new();
    nodes.insert(null_u256(), null_h256());

    for level in 1..depth {
        let next_level = level.sub(1);
        let mut left = nodes.get(&U256::from(next_level)).unwrap().to_vec();
        let mut right = left.clone();
        left.append(&mut right);
        let hash = keccak256(&left);
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
    fn create_tree() {
        let hex = "0x0101010101010101010101010101010101010101010101010101010101010101";
        let exp_root = "0x48ce19d92fe8d6b4be1d7744c1a798bde5d7f12ad192fe520aeae0462f3df29e";

        let leaves: HashMap<U256, H256> = [
            (U256::from(1), H256::from(hex)),
            (U256::from(2), H256::from(hex)),
            (U256::from(3), H256::from(hex)),
            (U256::from(4), H256::from(hex)),
        ]
        .iter()
        .cloned()
        .collect();

        let tree = MerkleTree::new(leaves, DEFAULT_DEPTH);
        assert_eq!(tree.root(), H256::from(exp_root));
    }
}
