use std::marker::PhantomData;

use crate::{app::App, arena::Arena, view::View, widget::{Widget}, handle::{UntypedHandle, TypedHandle, WidgetHandleTrait, ParentNode}, context::IndigoContext};

// //&'a mut IndigoContext<'a, A, V>,
// type UiIterItemRef<'a, A, V> = (&'a Box<dyn Widget<A, V>>, &'a Vec<usize>, &'a Option<usize>);
// type UiIterItemMut<'a, A, V> = (&'a mut Box<dyn Widget<A, V>>, &'a mut Vec<usize>, &'a mut Option<usize>);

// pub struct UiTreeIter<'a, A, V> 
// where 
//     A: App,
//     V: View<A>
// {
//     widget_arena: &'a Arena<WidgetWrapper<A, V>>,
//     i: usize,
// }

// impl<'a, A, V> Iterator for UiTreeIter<'a, A, V> 
// where 
//     A: App,
//     V: View<A>
// {
//     type Item = &'a WidgetWrapper<A, V>;
//     fn next(&mut self) -> Option<Self::Item> {
//         while self.i < self.widget_arena.vec.len() {
//             self.i += 1;
//             if let Some(wrapper) = self.widget_arena.get(self.i) {
//                 return Some(wrapper);
//             }
//         }
//         None
//     }
// }

// pub struct UiTreeIterMut<'a, A, V>
// where 
//     A: App,
//     V: View<A>
// {
//     widget_arena: &'a mut Arena<WidgetWrapper<A, V>>,
//     // children_arena: &'a mut Arena<Vec<usize>>,
//     // parent_arena: &'a mut Arena<Option<usize>>,
//     i: usize,
// }


// impl<'a, A, V> Iterator for UiTreeIterMut<'a, A, V>
// where 
//     A: App,
//     V: View<A>
// {
//     type Item = &'a mut WidgetWrapper<A, V>;
//     fn next(&mut self) -> Option<Self::Item> {
//         while self.i < self.widget_arena.vec.len() {
//             self.i += 1;
            
//             // SAFETY: Bla bla compiler doesnt know that these can't alias because they are subsequent calls
//             // adapted from LRUCacheMutIterator from this crate:
//             // https://docs.rs/uluru/0.1.0/src/uluru/lib.rs.html#210-214 
//             unsafe {
//                 if let Some(wrapper) = self.widget_arena.get(self.i) {
//                     // let children = self.children_arena.get(self.i).unwrap();
//                     // let parent = self.parent_arena.get(self.i).unwrap();

//                     let wrapper = &mut *(wrapper as *const WidgetWrapper<A, V> as *mut _);

//                     // let widget = &mut *(wrapper.get_widget_ref() as *const dyn Widget<A, V> as *mut _);
//                     // let children = &mut *(wrapper.get_children_ref() as *const Vec<usize> as *mut _);
//                     // let parent = &mut *(wrapper.get_parent_ref() as *const Option<usize> as *mut _);

//                     return Some(wrapper);
//                 }
//             }
//         }
//         None
//     }
// }

pub struct UiTree<A, V>
where 
    A: App,
    V: View<A>
{
    pub widget_arena: Arena<Box<dyn Widget<A, V>>>,
    pub children_arena: Arena<Vec<UntypedHandle>>,
    pub parent_arena: Arena<ParentNode>,
    pub(crate) pending_init: Vec<UntypedHandle>,
}

// impl<'a, A, V> IntoIterator for &'a UiTree<A, V> 
// where 
//     A: App,
//     V: View<A>
// {
//     type Item = &'a WidgetWrapper<A, V>;
//     type IntoIter = UiTreeIter<'a, A, V>;
//     fn into_iter(self) -> Self::IntoIter {
//         UiTreeIter {
//             widget_arena: &self.widget_arena,
//             // children_arena: &self.children_arena,
//             // parent_arena: &self.parent_arena,
//             i: 0
//         }
//     }
// }

// impl<'a, A, V> IntoIterator for &'a mut UiTree<A, V> 
// where 
//     A: App,
//     V: View<A>,
// {
//     type Item = &'a mut WidgetWrapper<A, V>;
//     type IntoIter = UiTreeIterMut<'a, A, V>;
//     fn into_iter(self) -> Self::IntoIter {
//         UiTreeIterMut {
//             widget_arena: &mut self.widget_arena,
//             // children_arena: &mut self.children_arena,
//             // parent_arena: &mut self.parent_arena,
//             i: 0
//         }
//     }
// }

impl<A, V> Default for UiTree<A, V>
where
    A: App,
    V: View<A> 
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

impl<A, V> UiTree<A, V> 
where
    A: App,
    V: View<A>
{
    /// Inserts a `dyn Widget<A, V>` into a tree `UiTree<A, V>` returning a typed handle which can be used to retrieve the initial object
    /// # Example
    /// ```rust
    /// let mut ui_tree = UiTree::<A, V>::new();
    /// ui_tree.insert(
    ///     ExampleWidget {},  // ExampleWidget must implement `Widget<A, V>`
    ///     None::<TypedHandle<()>>
    /// );
    /// ```  
    pub fn insert<T, P>(&mut self, widget: T, parent_enum: P) -> TypedHandle<T>
    where
        T: Widget<A, V>,
        P: Into<ParentNode>
    {        
        let parent_enum = parent_enum.into();

        // Insert the new widget and get it's assigned index
        let insertion_idx = self.widget_arena.insert(
            Box::new(widget)
        );

        self.children_arena.insert(Vec::new());
        self.parent_arena.insert(parent_enum);

        if let ParentNode::Handle(parent_handle) = parent_enum {
            let parents_children = self.children_arena.get_mut(parent_handle.index).unwrap();
            parents_children.push(UntypedHandle {
                index: insertion_idx
            });
        } 

        self.pending_init.push(UntypedHandle { index: insertion_idx });

        TypedHandle {
            _marker: PhantomData,
            index: insertion_idx,
        }
    }

    pub fn remove<W>(&mut self, handle: impl Into<UntypedHandle>)
    where
        W: Widget<A, V>,
    {
        let handle = handle.into();

        if let Some(_) = self.widget_arena.remove(handle.index) {
            let removed_parent_enum = self.parent_arena.remove(handle.index).unwrap(); 
            //This node's children's parent is changed to the removed node parent
            // Parent > Removed node > Children
            //         vvvvv
            // Parent > Children
            
            //Append the removed node's children to their parents
            let mut removed_children = self.children_arena.remove(handle.index).unwrap();
            removed_children
                .iter()
                .for_each(|child_handle| {
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

        // if let Some(parent_idx) = node_parent {
        //     let parents_children = self.children_arena.get_mut(parent_idx).unwrap();
        //     parents_children.drain_filter(|&mut c| c == index);
        //     parents_children.append(&mut node_children);
        // }
    }

    // pub fn get<W: Into<WidgetHandle>>(&self, handle: W) -> Option<&WidgetWrapper<A, V>> {
    //     let handle: WidgetHandle = handle.into();
    //     self.widget_arena.get(handle.index)
    // }
    
    // pub fn get_mut<W: Into<WidgetHandle>>(&mut self, handle: W) -> Option<&mut WidgetWrapper<A, V>> {
    //     let handle: WidgetHandle = handle.into();
    //     self.widget_arena.get_mut(handle.index)
    // }

    /*
    // NOTE: potentailly buggy

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


 /*
    /// Runs the `closure` for each widget in the tree passing in a &mut to itself (tree), a & to the current widget as well as its index
    /// # Example
    /// ```rust
    /// let ui_tree = UiTree::<A, V>::new();
    /// ui_tree.for_each_ref(
    ///     |
    ///         ui_tree: &mut UiTree<A, V>,
    ///         widget: &Box<dyn Widget<A, V>>,
    ///         idx: usize
    ///     | { ... }
    /// );
    /// ```
    pub(crate) fn for_each_ref<F>(&mut self, closure: F)
    where
        F: Fn(&mut Self, &Box<dyn WidgetWrapperTrait<A, V>>, usize),
    {
        for index in 0..self.widget_arena.vec.len() {
            if self.widget_arena.vec[index].is_some() {
                let widget = self.widget_arena.vec[index].as_ref().unwrap();                

                closure(self, widget, index);
            }
        }
    }

    /// Runs the `closure` for each widget in the tree passing in a &mut itself (tree), a &mut to the current widget as well as its index
    /// # Example
    /// ```rust
    /// let ui_tree = UiTree::<A, V>::new();
    /// ui_tree.for_each_mut(
    ///     |
    ///         ui_tree: &mut UiTree<A, V>,
    ///         widget: &mut Box<dyn Widget<A, V>>,
    ///         idx: usize
    ///     | { ... }
    /// );
    /// ```
    pub(crate) fn for_each_mut<F>(&mut self, mut closure: F)
    where
        F: FnMut(&mut Self, &mut Box<dyn WidgetWrapperTrait<A, V>>, usize),
    {
        for index in 0..self.widget_arena.vec.len() {
            if self.widget_arena.vec[index].is_some() {
                let widget = self.widget_arena.vec[index].as_mut().unwrap();

                closure(self, widget, index);

            }
        }
    } */


    // NOTE: Perhaps these should return either `Some(&[mut] T)`, `None` or `Err` since the index
    // could exist but be moved out temporarily

    /// Returns an `Ok(&T)` if the widget pointed at by the handle exists and is the correct type
    ///
    /// Returns an `IndigoError::InvalidHandle` if the widget doesnt exist or is the wrong type
    /// # Example
    /// ```rust
    /// let handle: WidgetHandle<ExampleWidget> = ui_tree.insert(ExampleWidget {});
    ///
    /// match ui_tree.get_ref(&handle) {
    ///     Ok::<&ExampleWidget>(widget) => widget.do_something(),
    ///     IndigoError::InvalidHandle => eprintln!("Invalid handle: {reason}"),
    ///     _ => unreachable!()
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn get_untyped_ref(&self, handle: impl WidgetHandleTrait) -> Option<&Box<dyn Widget<A, V>>>
    {
        self.widget_arena.vec[handle.index()].as_ref()
    }

    #[inline]
    #[must_use]
    pub fn get_typed_ref<W: Widget<A, V>>(&self, handle: &TypedHandle<W>) -> Option<&W>
    {
        self.get_untyped_ref(handle)?.downcast_ref::<W>()
    }

    /// Returns an `Ok(&mut T)` if the widget pointed at by the handle exists and is the correct type
    ///
    /// Returns an `IndigoError::InvalidHandle` if the widget doesnt exist or is the wrong type
    /// # Example
    /// ```rust
    /// let handle: WidgetHandle<ExampleWidget> = ui_tree.insert(ExampleWidget {});
    ///
    /// match ui_tree.get_mut(&handle) {
    ///     Ok::<&mut ExampleWidget>(widget) => widget.do_something(),
    ///     IndigoError::InvalidHandle { reason } => eprintln!("Invalid handle: {reason}"),
    ///     _ => unreachable!()
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn get_untyped_mut(&mut self, handle: impl WidgetHandleTrait) -> Option<&mut Box<dyn Widget<A, V>>> 
    {
        self.widget_arena.vec[handle.index()].as_mut()
    }

    #[inline]
    #[must_use]
    pub fn get_typed_mut<W: Widget<A, V>>(&mut self, handle: &TypedHandle<W>) -> Option<&mut W>
    {
        self.get_untyped_mut(handle)?.downcast_mut::<W>()
    }
    // // pub fn run_for_ref<T, F, Y>(&mut self, handle: &TypedHandle<T>, closure: F) -> Y
    // where
    //     T: Widget<A, V>,
    //     F: FnOnce(&mut Self, &T) -> Y
    // {
    //     closure(self, handle)
    // }

    // pub fn run_for_mut<T, F, Y>(&mut self, handle: &mut TypedHandle<T>, closure: F) -> Y 
    // where
    //     T: Widget<A, V>,
    //     F: FnOnce(&mut Self, &mut T) -> Y
    // {
    //     closure(self, handle)
    // }


}
