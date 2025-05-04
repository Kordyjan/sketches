use std::mem;

use smallvec::SmallVec;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct SparseVec<const CAP: usize, T> {
    mask: usize,
    data: SmallVec<[T; 4]>,
}

impl<const CAP: usize, T> Default for SparseVec<CAP, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize, T> SparseVec<CAP, T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            mask: 0,
            data: SmallVec::new(),
        }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.mask.count_ones() as usize
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mask == 0
    }

    pub fn insert(&mut self, pos: usize, elem: T) {
        let real_pos = self.elems_before(pos);
        let already_present = self.mask & (1 << (CAP - pos - 1)) != 0;
        if already_present {
            self.data[real_pos] = elem;
        } else {
            self.mask |= 1 << (CAP - pos - 1);
            self.data.insert(real_pos, elem);
        }
    }

    #[must_use]
    pub fn get(&self, pos: usize) -> Option<&T> {
        if self.mask & (1 << (CAP - pos - 1)) != 0 {
            Some(&self.data[self.elems_before(pos)])
        } else {
            None
        }
    }

    pub fn remove(&mut self, pos: usize) -> Option<T> {
        if self.mask & (1 << (CAP - pos - 1)) != 0 {
            let real_pos = self.elems_before(pos);
            let res = Some(self.data.remove(real_pos));
            self.mask &= !(1 << (CAP - pos - 1));
            res
        } else {
            None
        }
    }

    #[must_use]
    pub fn swap(&mut self, pos: usize, elem: T) -> Option<T> {
        if self.mask & (1 << (CAP - pos - 1)) != 0 {
            let real_pos = self.elems_before(pos);
            let mut res = elem;
            mem::swap(&mut self.data[real_pos], &mut res);
            Some(res)
        } else {
            self.insert(pos, elem);
            None
        }
    }

    pub fn keys(&self) -> Vec<usize> {
        (0..CAP)
            .filter(|&i| self.mask & (1 << (CAP - i - 1)) != 0)
            .collect()
    }

    pub fn iter(&self) -> Iter<'_, CAP, T> {
        Iter {
            index: 0,
            vector: self,
        }
    }

    fn elems_before(&self, pos: usize) -> usize {
        (self.mask >> (CAP - pos)).count_ones() as usize
    }
}

impl<const CAP: usize, T: Clone> Clone for SparseVec<CAP, T> {
    fn clone(&self) -> Self {
        Self {
            mask: self.mask,
            data: self.data.clone(),
        }
    }
}

impl<'a, const CAP: usize, T> IntoIterator for &'a SparseVec<CAP, T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, CAP, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a, const CAP: usize, T> {
    index: usize,
    vector: &'a SparseVec<CAP, T>,
}

impl<'a, const CAP: usize, T> Iterator for Iter<'a, CAP, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.vector.len() {
            None
        } else {
            let res = Some(&self.vector.data[self.index]);
            self.index += 1;
            res
        }
    }
}
