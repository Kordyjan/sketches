use std::marker::PhantomData;

pub struct Tripper<T, NI, LI>
where
    T: ?Sized,
{
    phantom_data: PhantomData<T>,
    stack: Vec<NI>,
    leaves: Option<LI>,
}

pub trait TreeIterable<'a> {
    type Node;
    type Leaf;
    type NodesIterator: Iterator<Item = Self::Node>;
    type LeavesIterator: Iterator<Item = Self::Leaf>;

    fn layer_iterator(n: &'a Self::Node) -> Layer<Self::NodesIterator, Self::LeavesIterator>;

    fn root(&'a self) -> &'a Self::Node;

    fn tree_iterator(&'a self) -> Tripper<Self, Self::NodesIterator, Self::LeavesIterator> {
        let root = Self::layer_iterator(self.root());
        match root {
            Layer::Nodes(ni) => Tripper {
                stack: vec![ni],
                leaves: None,
                phantom_data: PhantomData,
            },
            Layer::Leaves(li) => Tripper {
                stack: vec![],
                leaves: Some(li),
                phantom_data: PhantomData,
            },
        }
    }
}

impl<'a, T: TreeIterable<'a>> Iterator for &'a mut Tripper<T, T::NodesIterator, T::LeavesIterator> {
    type Item = T::Leaf;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.leaves {
            Some(ref mut it) => match it.next() {
                Some(e) => Some(e),
                None => self.progress_stack(),
            },
            None => self.progress_stack(),
        }
    }
}

impl<'a, T, NI, LI, N, L> Tripper<T, NI, LI>
where
    T: TreeIterable<'a, NodesIterator = NI, LeavesIterator = LI, Node = N, Leaf = L>,
    NI: Iterator<Item = N>,
    LI: Iterator<Item = L>,
{
    fn progress_stack(&mut self) -> Option<L> {
        loop {
            let top = self.stack.last_mut()?;
            let next = match top.next() {
                None => Step::Pop,
                Some(node) => match T::layer_iterator(&node) {
                    Layer::Nodes(ni) => Step::Push(ni),
                    Layer::Leaves(li) => Step::Ret(li),
                },
            };

            match next {
                Step::Pop => {
                    self.stack.pop();
                }
                Step::Push(ni) => self.stack.push(ni),
                Step::Ret(mut li) => {
                    let next_leaf = li.next();
                    if next_leaf.is_some() {
                        self.leaves = Some(li);
                        break next_leaf;
                    }
                }
            }
        }
    }
}

enum Step<N, L> {
    Pop,
    Push(N),
    Ret(L),
}

pub enum Layer<NI, LI> {
    Nodes(NI),
    Leaves(LI),
}
