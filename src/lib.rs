extern crate tiny_keccak;

use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::ops::Sub;
use std::vec::Vec;

use tiny_keccak::keccak256;

type Node = HashMap<usize, Box<[u8; 32]>>;

pub const DEFAULT_DEPTH: usize = 257;

pub struct Tree {
    data: RefCell<Vec<Node>>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            data: RefCell::new(Vec::new()),
        }
    }

    pub fn add_branch(&self, branch: Node) {
        self.data.borrow_mut().push(branch)
    }

    pub fn leaf_from_hash(&self, branch: usize, key: usize, val: Box<[u8; 32]>) {
        self.data.borrow_mut()[branch].insert(key, val);
    }

    pub fn get_leaf(&self, branch: usize, key: usize, result: &mut Vec<u8>) {
        *result = self.data.borrow()[branch].get(&key).unwrap().to_vec();
    }

    pub fn len(&self) -> usize {
        self.data.borrow().len()
    }
}

pub fn create_default_nodes(depth: usize) -> Node {
    let mut default_nodes: Node = Node::new();
    default_nodes.insert(0, Box::new([0_u8; 32]));

    for level in 1..depth {
        let next_level = level.sub(1);
        let mut prev_default = default_nodes.get(&next_level).unwrap().to_vec();
        let mut prev_default2 = prev_default.clone();

        prev_default.append(&mut prev_default2);

        default_nodes.insert(level, Box::new(keccak256(&prev_default)));
    }
    default_nodes
}

pub fn sort_keys(nodes: &Node) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::new();

    for (k, _) in nodes {
        result.push(*k)
    }
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    extern crate tiny_keccak;

    use super::*;

    fn new_tree() -> Tree {
        let tree = Tree::new();
        tree.add_branch(HashMap::new());
        tree
    }

    #[test]
    fn add_branch() {
        let tree = new_tree();

        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn leaf_from_hash() {
        let test_branch = 0;
        let test_key = 100;
        let exp = [1_u8; 32];
        let tree = new_tree();

        tree.leaf_from_hash(test_branch, test_key, Box::new(exp));
        let mut res: Vec<u8> = vec![];
        tree.get_leaf(test_branch, test_key, &mut res);

        assert_eq!(res, exp);
    }

    #[test]
    fn create_default_nodes() {
        let exp = super::DEFAULT_DEPTH;
        let default_nodes = super::create_default_nodes(exp);
        assert_eq!(default_nodes.len(), exp);
    }
}
