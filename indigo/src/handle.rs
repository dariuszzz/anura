use std::marker::PhantomData;

pub trait AsUntypedHandle {
    fn handle(&self) -> UntypedHandle;
}

#[derive(Copy, Eq, PartialEq)]
pub struct TypedHandle<T> {
    pub(crate) _marker: PhantomData<T>,
    pub(crate) index: usize,
}

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
            "{}<{}>",
            "TypedHandle",
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

// impl<T> WidgetHandleTrait for TypedHandle<T> {
//     fn index(&self) -> usize {
//         self.index
//     }
// }

// impl<T> WidgetHandleTrait for &TypedHandle<T> {
//     fn index(&self) -> usize {
//         self.index
//     }
// }

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UntypedHandle {
    pub(crate) index: usize,
}

// impl WidgetHandleTrait for UntypedHandle {
//     fn index(&self) -> usize {
//         self.index
//     }
// }

// impl WidgetHandleTrait for &UntypedHandle {
//     fn index(&self) -> usize {
//         self.index
//     }
// }

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
pub enum ParentNode {
    Handle(UntypedHandle),
    Root,
}

impl<T: AsUntypedHandle> From<T> for ParentNode {
    fn from(value: T) -> Self {
        ParentNode::Handle(value.handle())
    }
}