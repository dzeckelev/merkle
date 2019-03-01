extern crate ethereum_types;
extern crate tiny_keccak;

use std::collections::hash_map::HashMap;
use std::error::Error;
use std::ops::Sub;
use std::rc::Rc;
use std::vec::Vec;

use ethereum_types::{H256, U256};
use std::fmt;
use tiny_keccak::keccak256;

pub const DEFAULT_DEPTH: usize = 257;

pub type Nodes = HashMap<U256, H256>;
pub type Levels = Vec<Nodes>;
pub type Result<T> = std::result::Result<T, TreeError>;

#[derive(Debug)]
pub struct Tree {
    root: H256,
    depth: usize,
    default_nodes: Rc<Nodes>,
    tree: Rc<Levels>,
}

#[derive(Debug, Clone)]
pub enum TreeError {
    ErrSmallDepth,
    ErrKeyNotFound,
}

impl Error for TreeError {
    fn description(&self) -> &str {
        match *self {
            TreeError::ErrSmallDepth => "small depth",
            TreeError::ErrKeyNotFound => "key not found",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TreeError::ErrSmallDepth => None,
            TreeError::ErrKeyNotFound => None,
        }
    }
}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TreeError::ErrSmallDepth => write!(f, "small depth"),
            TreeError::ErrKeyNotFound => write!(f, "key not found"),
        }
    }
}

impl Tree {
    pub fn build(nodes: Nodes, depth: usize) -> Result<Tree> {
        let len = nodes.len();
        let cap = 2.0_f64.powi(depth as i32);

        if (len as f64) > cap {
            return Err(TreeError::ErrSmallDepth);
        }

        let default_nodes = default_nodes(depth)?;
        let levels = create_tree_levels(nodes, &default_nodes, depth)?;
        let root = root(&levels)?;

        let mt = Tree {
            root,
            depth,
            default_nodes: Rc::new(default_nodes),
            tree: Rc::new(levels),
        };

        Ok(mt)
    }

    pub fn create_proof(&self, index: U256) -> Result<Vec<u8>> {
        let mut index = index;
        let mut proof = Vec::new();

        for level_index in 0..self.depth - 1 {
            let sub_index: U256;

            if index % 2 == null_u256() {
                sub_index = index + 1;
            } else {
                sub_index = index - 1;
            }

            index = index.clone() / 2;

            let tree = self.tree.clone();
            let level: &Nodes = tree.get(level_index).unwrap();

            let mut data = level.get(&sub_index).map_or_else(
                || {
                    let default_nodes = self.default_nodes.clone();
                    default_nodes
                        .get(&U256::from(level_index))
                        .unwrap_or(&null_h256())
                        .to_vec()
                },
                |hash| hash.to_vec(),
            );

            proof.append(&mut data);
        }

        Ok(proof)
    }

    pub fn verify_proof(&self, index: U256, leaf: H256, root: H256, proof: &[u8]) -> bool {
        verify_proof(index, leaf, root, proof)
    }

    pub fn root(&self) -> H256 {
        self.root.clone()
    }
}

pub fn verify_proof(index: U256, leaf: H256, root: H256, proof: &[u8]) -> bool {
    if proof.len() == 0 || proof.len() % 32 != 0 {
        return false;
    }

    let mut index = index;
    let mut computed_hash: Vec<u8> = leaf.to_vec();
    let mut count = 0;
    let step = 32;

    for _ in proof.iter().step_by(step) {
        let mut proof_element = proof[count..count + step].to_vec();

        if index % 2 == null_u256() {
            computed_hash.append(&mut proof_element);
            computed_hash = keccak256(computed_hash.as_slice()).to_vec();
        } else {
            proof_element.append(&mut computed_hash);
            computed_hash = keccak256(proof_element.as_slice()).to_vec();
        }
        index = index / 2;
        count = count + step;
    }

    computed_hash == root.to_vec()
}

fn root(levels: &Levels) -> Result<H256> {
    let level: &Nodes = levels
        .get(levels.len() - 1)
        .ok_or(TreeError::ErrKeyNotFound)?;
    let hash = level
        .get(&null_u256())
        .unwrap_or(&null_h256())
        .clone()
        .into();
    Ok(hash)
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
        left = get_value(tree_level, node_index);
        right = get_value(default_nodes, &U256::from(level));
    } else {
        if node_index.clone() == prev_index + 1 {
            left = get_value(tree_level, prev_index);
            right = get_value(tree_level, node_index);
        } else {
            left = get_value(default_nodes, &U256::from(level));
            right = get_value(tree_level, node_index);
        }
    }
    calc_hash(&mut left, &mut right)
}

fn fill_tree(tree: &mut Levels, default_nodes: &Nodes, depth: usize) -> Result<()> {
    for level in 0..depth - 1 {
        let mut next_level: Nodes = HashMap::new();
        let mut prev_index = null_u256();
        let mut keys;

        {
            let lvl = tree.get(level).ok_or(TreeError::ErrKeyNotFound)?;
            keys = sort_keys(lvl);
        }

        for node_index in keys {
            {
                let tree_level: &Nodes = tree.get(level).ok_or(TreeError::ErrKeyNotFound)?;
                let div = node_index / 2;
                let mut hash =
                    create_hash(level, &prev_index, &node_index, default_nodes, tree_level);

                next_level.insert(U256::from(div), H256::from(hash));
            }
            prev_index = node_index.clone();
        }

        tree.push(next_level);
    }

    Ok(())
}

fn create_tree_levels(nodes: Nodes, default_nodes: &Nodes, depth: usize) -> Result<Levels> {
    let mut levels = vec![nodes];
    match fill_tree(&mut levels, &default_nodes, depth) {
        Ok(_) => Ok(levels),
        Err(e) => Err(e),
    }
}

fn default_nodes(depth: usize) -> Result<Nodes> {
    let mut nodes = HashMap::new();
    nodes.insert(null_u256(), null_h256());

    if depth < 2 {
        return Err(TreeError::ErrSmallDepth);
    }

    for level in 1..depth {
        let next_level = level.sub(1);
        let mut left = nodes
            .get(&U256::from(next_level))
            .ok_or(TreeError::ErrKeyNotFound)?
            .to_vec();
        let mut right = left.clone();
        left.append(&mut right);
        let hash = keccak256(&left);
        nodes.insert(U256::from(level), H256::from(hash));
    }

    Ok(nodes)
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

        let tree = Tree::build(leaves, DEFAULT_DEPTH).unwrap();
        assert_eq!(tree.root(), H256::from(exp_root));
    }
}
