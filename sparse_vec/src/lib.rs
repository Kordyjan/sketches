use std::mem;

#[cfg(test)]
mod tests;

pub struct SparseVec<const CAP: usize, T> {
    mask: usize,
    data: Vec<T>,
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
            data: Vec::with_capacity(2),
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
        self.mask |= 1 << (CAP - pos - 1);
        self.data.insert(real_pos, elem);
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
            None
        }
    }

    fn elems_before(&self, pos: usize) -> usize {
        (self.mask >> (CAP - pos)).count_ones() as usize
    }
}
