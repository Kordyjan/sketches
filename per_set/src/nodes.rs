use std::sync::Arc;

use smallvec::{smallvec, SmallVec};
use sparse_vec::SparseVec;

pub enum Node<K, V> {
    Leaf {
        data: SmallVec<[Arc<(K, V)>; 2]>,
        weight: usize,
    },
    Branch {
        data: SparseVec<16, Arc<Node<K, V>>>,
        weight: usize,
    },
}

impl<K, V> Node<K, V> {
    pub fn empty_branch() -> Self {
        Node::Branch {
            data: SparseVec::new(),
            weight: 0,
        }
    }

    fn allocate(key: K, value: V, address: BitShifter) -> Self {
        match address.shift() {
            None => Node::Leaf {
                data: smallvec![Arc::new((key, value))],
                weight: 1,
            },
            Some((new_address, index)) => {
                let mut data = SparseVec::new();
                data.insert(
                    index as usize,
                    Arc::new(Node::allocate(key, value, new_address)),
                );
                Node::Branch { data, weight: 1 }
            }
        }
    }

    pub fn weight(&self) -> usize {
        match self {
            Node::Leaf { weight, .. } => *weight,
            Node::Branch { weight, .. } => *weight,
        }
    }
}

impl<K, V> Default for Node<K, V> {
    fn default() -> Self {
        Node::empty_branch()
    }
}

impl<K: Eq, V> Node<K, V> {
    pub fn insert(
        node: &Arc<Node<K, V>>,
        key: K,
        value: V,
        address: BitShifter,
    ) -> Arc<Node<K, V>> {
        match &**node {
            Node::Leaf { data, weight } => {
                match data.into_iter().position(|arc| (**arc).0 == key) {
                    None => {
                        let mut new_data: SmallVec<_> = data.clone();
                        new_data.push(Arc::new((key, value)));
                        Arc::new(Node::Leaf {
                            data: new_data,
                            weight: weight + 1,
                        })
                    }
                    Some(pos) => {
                        let mut new_data: SmallVec<_> = data.clone();
                        new_data[pos] = Arc::new((key, value));
                        Arc::new(Node::Leaf {
                            data: new_data,
                            weight: *weight,
                        })
                    }
                }
            }
            Node::Branch { data, weight } => {
                let (new_address, index) = address.shift().unwrap();
                match data.get(index as usize) {
                    None => {
                        let new_node = Arc::new(Node::allocate(key, value, new_address));
                        let mut new_data = data.clone();
                        new_data.insert(index as usize, new_node);
                        Arc::new(Node::Branch {
                            data: new_data,
                            weight: weight + 1,
                        })
                    }
                    Some(next) => {
                        let new_node = Node::insert(next, key, value, new_address);
                        if Arc::ptr_eq(node, &new_node) {
                            node.clone()
                        } else {
                            let mut new_data = data.clone();
                            new_data.insert(index as usize, new_node);
                            Arc::new(Node::Branch {
                                data: new_data,
                                weight: weight + 1,
                            })
                        }
                    }
                }
            }
        }
    }
}

pub struct BitShifter {
    value: u64,
    shift: usize,
}

impl BitShifter {
    pub fn new(value: u64) -> Self {
        Self { value, shift: 0 }
    }

    fn shift(&self) -> Option<(BitShifter, u64)> {
        if self.shift < 64 {
            let res = self.value & 0b1111;
            Some((
                BitShifter {
                    value: self.value >> 4,
                    shift: self.shift + 4,
                },
                res,
            ))
        } else {
            None
        }
    }
}
