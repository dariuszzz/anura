use std::{marker::PhantomData, hash::Hash};

pub trait AsUntypedHandle: Hash {
    fn handle(&self) -> UntypedHandle;
}

#[derive(Copy, Eq, PartialEq)]
pub struct TypedHandle<T> {
    pub(crate) _marker: PhantomData<T>,
    pub(crate) index: usize,
}

impl<T> Hash for TypedHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

//Manually implement clone so handles of uncloneable widgets are still clone
impl<T> std::clone::Clone for TypedHandle<T> {
    fn clone(&self) -> Self {
        Self {
            _marker: PhantomData,
            index: self.index,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.index = source.index
    }
}

impl<T> std::fmt::Debug for TypedHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!(
            "TypedHandle<{}>",
            std::any::type_name::<T>().split("::").last().unwrap()
        ))
        .field("index", &self.index)
        .finish()
    }
}

impl<T> AsUntypedHandle for TypedHandle<T> {
    fn handle(&self) -> UntypedHandle {
        UntypedHandle { index: self.index }
    }
}

impl<T> AsUntypedHandle for &TypedHandle<T> {
    fn handle(&self) -> UntypedHandle {
        UntypedHandle { index: self.index }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct UntypedHandle {
    pub(crate) index: usize,
}

impl AsUntypedHandle for UntypedHandle{
    fn handle(&self) -> UntypedHandle {
        self.clone()
    }
}

impl AsUntypedHandle for &UntypedHandle{
    fn handle(&self) -> UntypedHandle {
        *self.clone()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NodeType {
    Handle(UntypedHandle),
    Root,
}

impl<T: AsUntypedHandle> From<T> for NodeType {
    fn from(value: T) -> Self {
        NodeType::Handle(value.handle())
    }
}