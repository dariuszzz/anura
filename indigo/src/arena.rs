use std::mem;

#[derive(Debug, Default)]
pub struct Arena<T> {
    pub vec: Vec<Option<T>>,
    pub free_spaces: Vec<usize>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            free_spaces: Vec::new(),
        }
    }

    pub fn overwrite(&mut self, index: usize, val: T) {
        _ = mem::replace(&mut self.vec[index], Some(val))
    }

    pub fn insert(&mut self, val: T) -> usize {
        match self.free_spaces.pop() {
            Some(idx) => {
                self.vec[idx] = Some(val);
                idx
            }
            None => {
                self.vec.push(Some(val));
                self.vec.len() - 1
            }
        }
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        self.free_spaces.push(idx);
        mem::replace(&mut self.vec[idx], None)
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        match self.vec.get(idx) {
            Some(v) => v.as_ref(),
            None => None,
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        match self.vec.get_mut(idx) {
            Some(v) => v.as_mut(),
            None => None,
        }
    }
}