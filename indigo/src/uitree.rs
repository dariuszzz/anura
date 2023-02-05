use std::marker::PhantomData;

use crate::{
    app::App,
    arena::Arena,
    handle::{NodeType, TypedHandle, UntypedHandle},
    prelude::{IndigoRenderer, AsUntypedHandle},
    view::View,
    widget::Widget,
};

pub struct UiTree<A, V, R> {
    pub widget_arena: Arena<Box<dyn Widget<A, V, R>>>,
    pub children_arena: Arena<Vec<UntypedHandle>>,
    pub parent_arena: Arena<NodeType>,
    pub(crate) pending_init: Vec<UntypedHandle>,
}

impl<A, V, R> Default for UiTree<A, V, R>
where
    A: App<R>,
    V: View<A, R>,
    R: IndigoRenderer,
{
    fn default() -> Self {
        Self {
            children_arena: Arena::new(),
            parent_arena: Arena::new(),
            widget_arena: Arena::new(),
            pending_init: Vec::new(),
        }
    }
}

impl<A, V, R> UiTree<A, V, R>
where
    A: App<R>,
    V: View<A, R>,
    R: IndigoRenderer,
{
    /// Returns a handle with no widget instance attached
    pub fn reserve_handle<T>(&mut self) -> TypedHandle<T> {
        let index = match self.widget_arena.free_spaces.pop() {
            Some(index) => index,
            None => {
                self.widget_arena.vec.push(None);
                self.parent_arena.vec.push(None);
                self.children_arena.vec.push(None);
                
                self.widget_arena.vec.len() - 1
            }
        };

        return TypedHandle {
            _marker: PhantomData,
            index
        }
        
    }

    /// Drops the widget previously referenced by the handle
    pub fn overwrite_handle<T, P>(&mut self, handle: &TypedHandle<T>, parent_enum: P, widget: T) 
    where
        T: Widget<A, V, R> + Default,
        P: Into<NodeType>
    {
        let parent_enum = parent_enum.into();
        let index = handle.index;

        self.widget_arena.overwrite(index, Box::new(widget));
        self.children_arena.overwrite(index, Vec::new());
        self.parent_arena.overwrite(index, parent_enum);

        if let NodeType::Handle(parent_handle) = parent_enum {
            let parents_children = self.children_arena.get_mut(parent_handle.index).unwrap();
            parents_children.push(UntypedHandle {
                index,
            });
        }

        self.pending_init.push(UntypedHandle {
            index,
        });
    }

    pub fn insert<T, P>(&mut self, widget: T, parent_enum: P) -> TypedHandle<T>
    where
        T: Widget<A, V, R> + Default,
        P: Into<NodeType>,
    {
        let handle = self.reserve_handle();
        self.overwrite_handle(&handle, parent_enum, widget);
        handle
    }

    pub fn remove<W>(&mut self, handle: impl Into<UntypedHandle>)
    where
        W: Widget<A, V, R>,
    {
        let handle = handle.into();

        if self.widget_arena.remove(handle.index).is_some() {
            let removed_parent_enum = self.parent_arena.remove(handle.index).unwrap();
            //This node's children's parent is changed to the removed node parent
            // Parent > Removed node > Children
            //         vvvvv
            // Parent > Children

            //Append the removed node's children to their parents
            let mut removed_children = self.children_arena.remove(handle.index).unwrap();
            removed_children.iter().for_each(|child_handle| {
                self.parent_arena.vec[child_handle.index] = Some(removed_parent_enum);
            });

            if let NodeType::Handle(parent_handle) = removed_parent_enum {
                // Remove the removed node from its parent's children vector
                let parents_children = self.children_arena.get_mut(parent_handle.index).unwrap();

                parents_children.append(&mut removed_children);

                //remove the removed widget from its parents children
                parents_children.drain_filter(|&mut c| c == handle);
            }
        }
    }

    #[must_use]
    pub fn get_children_handles(&self, handle: &impl AsUntypedHandle) -> Vec<UntypedHandle> {
        let handle = handle.handle();
        self.children_arena.vec[handle.index].clone().expect("Invalid handle")
    }

    #[must_use]
    pub fn get_all_handles(&self) -> impl Iterator<Item = UntypedHandle> + '_ {
        (0..self.widget_arena.vec.len())
            .into_iter()
            .filter_map(|index| match self.widget_arena.vec[index] {
                Some(_) => Some(UntypedHandle { index }),
                None => None
            })
    }

    /*
       // might come in useful

       /// Merges one `UiTree<A, V>` into another
       ///
       /// All widgets' indexes are offset by the length of the "absorbing" tree.
       ///
       /// e.g widget with an id of 2 will get assigned the id of absorbing_tree_len+2
       ///
       /// This moves out the entire tree and keeps all widgets inside of it intact
       /// # Example
       /// ```rust
       /// let mut tree_1 = UiTree::<A, V>::new();
       /// let tree_2 = UiTree::<A, V>::new();
       ///
       /// tree_1.merge(tree_2);
       /// ```
       pub(crate) fn merge(&mut self, mut other: UiTree<A, V>) {
           let other_len = other.widget_arena.vec.len();
           let self_len = self.widget_arena.vec.len();

           for idx in 0..other_len {
               if other.widget_arena.vec[idx].is_some() {
                   let widget = other.widget_arena.vec[idx].take().unwrap();
                   let mut children = other.children_arena.vec[idx].take().unwrap();
                   let mut parent = other.parent_arena.vec[idx].take().unwrap();

                   // offset children indexes  by this tree's length so all widget's children/parents are still valid
                   for c in &mut children {
                       *c += self_len;
                   }

                   // same for parent index
                   if let Some(parent_idx) = &mut parent {
                       *parent_idx += self_len;
                   }

                   self.widget_arena.insert(widget);
                   self.children_arena.insert(children);
                   self.parent_arena.insert(parent);
               }
           }
       }
    */

    #[inline]
    #[must_use]
    pub fn get_untyped_ref(
        &self,
        handle: &impl AsUntypedHandle,
    ) -> Option<&dyn Widget<A, V, R>> {
        self.widget_arena.vec[handle.handle().index].as_deref()
    }

    #[inline]
    #[must_use]
    pub fn get_typed_ref<W: Widget<A, V, R>>(&self, handle: &TypedHandle<W>) -> Option<&W> {
        (self.get_untyped_ref(handle)? as &dyn std::any::Any).downcast_ref::<W>()
    }

    #[inline]
    #[must_use]
    pub fn get_untyped_mut(
        &mut self,
        handle: &impl AsUntypedHandle,
    ) -> Option<&mut dyn Widget<A, V, R>> {
        self.widget_arena.vec[handle.handle().index].as_deref_mut()
    }

    #[inline]
    #[must_use]
    pub fn get_typed_mut<W: Widget<A, V, R>>(&mut self, handle: &TypedHandle<W>) -> Option<&mut W> {
        (self.get_untyped_mut(handle)? as &mut dyn std::any::Any).downcast_mut::<W>()
    }

    #[inline]
    pub(crate) fn run_on_moved_out<F, T>(&mut self, handle: &impl AsUntypedHandle, f: F) -> T 
    where
        F: FnOnce(&mut Self, &mut Box<dyn Widget<A, V, R>>) -> T,
    {
        let handle = handle.handle();
        let mut widget = self.widget_arena.vec[handle.index].take().unwrap();

        let res = f(self, &mut widget);

        self.widget_arena.vec[handle.index] = Some(widget);

        res
    }
}
