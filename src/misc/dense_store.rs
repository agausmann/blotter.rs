use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

pub struct DenseStore<T> {
    items: Vec<T>,
}

impl<T> DenseStore<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, index: Index<T>) -> Option<&T> {
        self.items.get(index.into_raw())
    }

    pub fn get_mut(&mut self, index: Index<T>) -> Option<&mut T> {
        self.items.get_mut(index.into_raw())
    }

    pub fn insert(&mut self, item: T) -> Index<T> {
        let index = Index::from_raw(self.items.len());
        self.items.push(item);
        index
    }

    #[must_use = "DenseStore::remove() renames an index; all external references must be replaced"]
    pub fn remove(&mut self, index: Index<T>) -> Option<(T, Rename<T>)> {
        let raw = index.into_raw();
        if self.items.get(raw).is_some() {
            let removed = self.items.swap_remove(raw);
            Some((
                removed,
                Rename {
                    src: Index::from_raw(self.items.len()),
                    dest: index,
                },
            ))
        } else {
            None
        }
    }
}

pub struct Index<T>(usize, PhantomData<T>);

impl<T> Index<T> {
    pub fn from_raw(raw: usize) -> Self {
        Self(raw, PhantomData)
    }

    pub fn into_raw(self) -> usize {
        self.0
    }
}

impl<T> Debug for Index<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Index").field(&self.0).finish()
    }
}

impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for Index<T> {}

impl<T> PartialEq for Index<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Index<T> {}

impl<T> Hash for Index<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub struct Rename<T> {
    pub src: Index<T>,
    pub dest: Index<T>,
}
