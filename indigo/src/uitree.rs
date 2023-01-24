use std::marker::PhantomData;

use crate::{
    app::App,
    arena::Arena,
    handle::{ParentNode, TypedHandle, UntypedHandle, WidgetHandleTrait},
    prelude::IndigoRenderer,
    view::View,
    widget::Widget,
};

pub struct UiTree<A, V, R> {
    pub widget_arena: Arena<Box<dyn Widget<A, V, R>>>,
    pub children_arena: Arena<Vec<UntypedHandle>>,
    pub parent_arena: Arena<ParentNode>,
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
    pub fn insert<T, P>(&mut self, widget: T, parent_enum: P) -> TypedHandle<T>
    where
        T: Widget<A, V, R>,
        P: Into<ParentNode>,
    {
        let parent_enum = parent_enum.into();

        // Insert the new widget and get it's assigned index
        let insertion_idx = self.widget_arena.insert(Box::new(widget));

        self.children_arena.insert(Vec::new());
        self.parent_arena.insert(parent_enum);

        if let ParentNode::Handle(parent_handle) = parent_enum {
            let parents_children = self.children_arena.get_mut(parent_handle.index).unwrap();
            parents_children.push(UntypedHandle {
                index: insertion_idx,
            });
        }

        self.pending_init.push(UntypedHandle {
            index: insertion_idx,
        });

        TypedHandle {
            _marker: PhantomData,
            index: insertion_idx,
        }
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

            if let ParentNode::Handle(parent_handle) = removed_parent_enum {
                // Remove the removed node from its parent's children vector
                let parents_children = self.children_arena.get_mut(parent_handle.index).unwrap();

                parents_children.append(&mut removed_children);

                //remove the removed widget from its parents children
                parents_children.drain_filter(|&mut c| c == handle);
            }
        }
    }

    #[must_use]
    pub fn get_all_handles(&self) -> Vec<UntypedHandle> {
        self.widget_arena
            .vec
            .iter()
            .enumerate()
            .filter(|(_, w)| w.is_some())
            .map(|(index, _)| UntypedHandle { index })
            .collect()
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
        handle: impl WidgetHandleTrait,
    ) -> Option<&dyn Widget<A, V, R>> {
        self.widget_arena.vec[handle.index()].as_deref()
    }

    #[inline]
    #[must_use]
    pub fn get_typed_ref<W: Widget<A, V, R>>(&self, handle: &TypedHandle<W>) -> Option<&W> {
        self.get_untyped_ref(handle)?.downcast_ref::<W>()
    }

    #[inline]
    #[must_use]
    pub fn get_untyped_mut(
        &mut self,
        handle: impl WidgetHandleTrait,
    ) -> Option<&mut dyn Widget<A, V, R>> {
        self.widget_arena.vec[handle.index()].as_deref_mut()
    }

    #[inline]
    #[must_use]
    pub fn get_typed_mut<W: Widget<A, V, R>>(&mut self, handle: &TypedHandle<W>) -> Option<&mut W> {
        self.get_untyped_mut(handle)?.downcast_mut::<W>()
    }

    #[inline]
    pub(crate) fn run_on_moved_out<F>(&mut self, handle: impl WidgetHandleTrait, mut f: F)
    where
        F: FnMut(&mut Self, &mut Box<dyn Widget<A, V, R>>),
    {
        let mut widget = self.widget_arena.vec[handle.index()].take().unwrap();

        f(self, &mut widget);

        self.widget_arena.vec[handle.index()] = Some(widget);
    }
}
