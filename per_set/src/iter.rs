use core::slice;
use std::sync::Arc;

use crate::{nodes::Node, PerMap};

pub struct Iter<'a, K, V> {
    stack: Vec<sparse_vec::Iter<'a, 16, Arc<Node<K, V>>>>,
    leaves: Option<slice::Iter<'a, Arc<(K, V)>>>,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub fn new<S>(map: &'a PerMap<K, V, S>) -> Self {
        let bottom = match map.data.as_ref() {
            Node::Branch { data, .. } => data.iter(),
            Node::Leaf { .. } => unreachable!(),
        };

        Iter {
            stack: vec![bottom],
            leaves: None,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = &'a Arc<(K, V)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.leaves.as_mut().and_then(Iterator::next) {
            Some(leaf) => Some(leaf),
            None => loop {
                let top = self.stack.last_mut()?;
                let step: Step<'a, K, V> = match top.next() {
                    None => Step::Pop,
                    Some(next) => match next.as_ref() {
                        Node::Leaf { data, .. } => Step::Ret(data.iter()),
                        Node::Branch { data, .. } => Step::Push(data.iter()),
                    },
                };

                match step {
                    Step::Pop => {
                        self.stack.pop();
                    }
                    Step::Push(next) => self.stack.push(next),
                    Step::Ret(mut li) => {
                        let next_leaf = li.next();
                        if next_leaf.is_some() {
                            self.leaves = Some(li);
                            break next_leaf;
                        }
                    }
                }
            },
        }
    }
}

enum Step<'a, K, V> {
    Pop,
    Push(sparse_vec::Iter<'a, 16, Arc<Node<K, V>>>),
    Ret(slice::Iter<'a, Arc<(K, V)>>),
}
