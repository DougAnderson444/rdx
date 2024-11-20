use anyhow::Result;
use core::any::Any;
use std::collections::BTreeSet;

use wasm_component_layer::{
    AsContext as _, ResourceOwn, ResourceType, StoreContext, StoreContextMut,
};

#[derive(Debug, thiserror::Error)]
/// Errors returned by operations on `ResourceTable`
pub enum ResourceTableError {
    /// ResourceTable has no free keys
    #[error("ResourceTable is full")]
    Full,
    /// Resource not present in table
    #[error("Resource not present in table")]
    NotPresent,
    /// Resource present in table, but with a different type
    #[error("Resource present in table, but with a different type")]
    WrongType,
    /// Resource cannot be deleted because child resources exist in the table. Consult wit docs for
    /// the particular resource to see which methods may return child resources.
    #[error("Resource cannot be deleted because child resources exist in the table")]
    HasChildren,

    /// From anyhow
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

/// This structure tracks parent and child relationships for a given table entry.
///
/// Parents and children are referred to by table index. We maintain the
/// following invariants to prevent orphans and cycles:
/// * parent can only be assigned on creating the entry.
/// * parent, if some, must exist when creating the entry.
/// * whenever a child is created, its index is added to children.
/// * whenever a child is deleted, its index is removed from children.
/// * an entry with children may not be deleted.
#[derive(Debug)]
struct TableEntry {
    /// The entry in the table, as a boxed dynamically-typed object
    entry: Box<dyn Any + Send>,
    /// The index of the parent of this entry, if it has one.
    parent: Option<u32>,
    /// The indices of any children of this entry.
    children: BTreeSet<u32>,
}

impl TableEntry {
    fn new(entry: Box<dyn Any + Send>, parent: Option<u32>) -> Self {
        Self {
            entry,
            parent,
            children: BTreeSet::new(),
        }
    }
    fn add_child(&mut self, child: u32) {
        debug_assert!(!self.children.contains(&child));
        self.children.insert(child);
    }
    fn remove_child(&mut self, child: u32) {
        let was_removed = self.children.remove(&child);
        debug_assert!(was_removed);
    }
}
/// The `ResourceTable` type maps a `Resource<T>` to its `T`.
#[derive(Debug, Default)]
pub struct ResourceTable {
    entries: Vec<TableEntry>,
    free_head: Option<usize>,
}

impl ResourceTable {
    /// Create an empty table
    pub fn new() -> Self {
        ResourceTable {
            entries: Vec::new(),
            free_head: None,
        }
    }

    /// Inserts a new value `T` into this table, returning a corresponding
    /// `Resource<T>` which can be used to refer to it after it was inserted.
    pub fn push<T>(&mut self, entry: T) -> Result<u32, ResourceTableError>
    where
        T: Send + Sync + 'static,
    {
        let idx = self.push_(TableEntry::new(Box::new(entry), None))?;
        Ok(idx)
    }

    /// Insert a resource at the next available index, and track that it has a
    /// parent resource.
    ///
    /// The parent must exist to create a child. All children resources must
    /// be destroyed before a parent can be destroyed - otherwise
    /// [`ResourceTable::delete`] will fail with
    /// [`ResourceTableError::HasChildren`].
    ///
    /// Parent-child relationships are tracked inside the table to ensure that
    /// a parent resource is not deleted while it has live children. This
    /// allows child resources to hold "references" to a parent by table
    /// index, to avoid needing e.g. an `Arc<Mutex<parent>>` and the associated
    /// locking overhead and design issues, such as child existence extending
    /// lifetime of parent referent even after parent resource is destroyed,
    /// possibility for deadlocks.
    ///
    /// Parent-child relationships may not be modified once created. There
    /// is no way to observe these relationships through the [`ResourceTable`]
    /// methods except for erroring on deletion, or the [`std::fmt::Debug`]
    /// impl.
    pub fn push_child<'a, T, S, E: wasm_runtime_layer::backend::WasmEngine>(
        &mut self,
        ctx: &'a mut StoreContextMut<S, E>,
        entry: T,
        parent: u32,
    ) -> Result<ResourceOwn>
    where
        T: Send + Sync + 'static + Copy,
    {
        let pollable_resource = ResourceOwn::new(ctx, entry, ResourceType::new::<T>(None))?;
        //let child = self.push_(TableEntry::new(Box::new(entry), Some(parent)))?;
        //self.entries[parent as usize].add_child(child);
        Ok(pollable_resource)
    }

    /// Push a new entry into the table, returning its handle. This will prefer to use free entries
    /// if they exist, falling back on pushing new entries onto the end of the table.
    fn push_(&mut self, e: TableEntry) -> Result<u32, ResourceTableError> {
        let ix = self
            .entries
            .len()
            .try_into()
            .map_err(|_| ResourceTableError::Full)?;
        self.entries.push(TableEntry::new(Box::new(e), None));
        Ok(ix)
    }
    /// Get an immutable reference to a resource of a given type at a given
    /// index.
    ///
    /// Multiple shared references can be borrowed at any given time.
    pub fn get<T: Any + Sized, E: wasm_runtime_layer::backend::WasmEngine>(
        &self,
        ctx: &StoreContext<T, E>,
        key: &ResourceOwn,
    ) -> Result<&T, ResourceTableError> {
        self.get_(*key.rep(ctx)?)?
            .downcast_ref()
            .ok_or(ResourceTableError::WrongType)
    }

    fn get_(&self, key: u32) -> Result<&dyn Any, ResourceTableError> {
        let r = self
            .entries
            .get(key as usize)
            .ok_or(ResourceTableError::NotPresent)?;
        Ok(&*r.entry)
    }

    /// Given the Pollable index, get the entry
    /// the return value of this fn gets passed into make_future closure
    pub(crate) fn get_any_mut(&mut self, index: u32) -> &mut dyn Any {
        // get parent sleep entry index from child pollable index
        let sleep_index = index as usize;

        // get the sleep entry
        let sleep_entry = &mut self.entries[sleep_index as usize];

        // return the entry
        let entry = sleep_entry.entry.as_mut();
        entry
    }
}
