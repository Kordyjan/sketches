use std::{
    borrow::Borrow,
    fmt::{self, Debug, Formatter},
    hash::Hash,
    sync::{atomic::AtomicUsize, Arc},
};

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

                Node::Branch {
                    data: data.clone(),
                    weight: 1,
                }
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
        let res = match &**node {
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
                        let mut new_data = data.clone();
                        let add_weight = new_node.weight();
                        new_data.insert(index as usize, new_node);
                        Arc::new(Node::Branch {
                            data: new_data,
                            weight: weight + add_weight - next.weight(),
                        })
                    }
                }
            }
        };
        res
    }

    pub fn get<Q>(&self, key: &Q, address: BitShifter) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        match self {
            Node::Leaf { data, .. } => data
                .iter()
                .find(|arc| arc.0.borrow() == key)
                .map(|arc| &arc.1),
            Node::Branch { data, .. } => {
                let (new_address, index) = address.shift().unwrap();
                data.get(index as usize)
                    .and_then(|node| node.get(key, new_address))
            }
        }
    }
}

impl<K: Debug, V: Debug> Debug for Node<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        inner_print(self, f, "")
    }
}

fn inner_print<K: Debug, V: Debug>(
    node: &Node<K, V>,
    f: &mut Formatter<'_>,
    prefix: &str,
) -> fmt::Result {
    match node {
        Node::Leaf { data, .. } => writeln!(f, "{}: {:?}", prefix, data),
        Node::Branch { data, .. } => {
            for i in data.keys().into_iter() {
                let new_prefix = format!("{} {:x}", prefix, i);
                inner_print(data.get(i).unwrap(), f, &new_prefix)?;
            }
            Ok(())
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

impl Debug for BitShifter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.shift < 64 {
            write!(f, "{:16x}", self.value)
        } else {
            write!(f, "X")
        }
    }
}
