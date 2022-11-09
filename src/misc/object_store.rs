use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::replace,
};

/// An object store that stores a free-list inline with its elements.
///
/// The original idea for this allocator is to have a stack of indexes that have
/// been vacated when items are removed from those locations. When adding an
/// item, the stack is checked first for a free index; if the stack is empty,
/// then the list has no vacant entries and the item is simply appended to the
/// end.
///
/// Instead of storing this stack separately; the cost of the free-list stack is
/// amortized by storing it inline with the items themselves. This may require
/// up to 8 bytes in addition to the size of each item.
pub struct ObjectStore<T> {
    first_vacant: usize,
    entries: Vec<Entry<T>>,
}

impl<T> ObjectStore<T> {
    pub fn new() -> Self {
        Self {
            first_vacant: usize::MAX,
            entries: Vec::new(),
        }
    }

    // pub fn contains(&self, address: Address<T>) -> bool {
    //     self.get(address).is_some()
    // }

    pub fn insert(&mut self, item: T) -> Address<T> {
        if let Some(entry) = self.entries.get_mut(self.first_vacant) {
            let address = Address::from_raw(self.first_vacant);
            let replaced = replace(entry, Entry::Occupied(item));
            match replaced {
                Entry::Vacant { next_vacant } => {
                    self.first_vacant = next_vacant;
                }
                _ => {
                    unreachable!("occupied entry in free list");
                }
            }
            address
        } else {
            let address = Address::from_raw(self.entries.len());
            self.entries.push(Entry::Occupied(item));
            address
        }
    }

    pub fn get(&self, address: Address<T>) -> Option<&T> {
        self.entries.get(address.into_raw()).and_then(Entry::get)
    }

    pub fn get_mut(&mut self, address: Address<T>) -> Option<&mut T> {
        self.entries
            .get_mut(address.into_raw())
            .and_then(Entry::get_mut)
    }

    pub fn remove(&mut self, address: Address<T>) -> Option<T> {
        let index = address.into_raw();

        match self.entries.get(index)? {
            Entry::Occupied(_) => {
                let replaced = replace(
                    &mut self.entries[index],
                    Entry::Vacant {
                        next_vacant: self.first_vacant,
                    },
                );
                self.first_vacant = index;
                match replaced {
                    Entry::Occupied(x) => Some(x),
                    _ => unreachable!("occupied is not occupied?"),
                }
            }
            Entry::Vacant { .. } => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Address<T>, &T)> {
        self.entries
            .iter()
            .enumerate()
            .flat_map(|(index, entry)| entry.get().map(|item| (Address::from_raw(index), item)))
    }

    // pub fn iter_mut(&mut self) -> impl Iterator<Item = (Address<T>, &mut T)> {
    //     self.entries
    //         .iter_mut()
    //         .enumerate()
    //         .flat_map(|(index, entry)| entry.get_mut().map(|item| (Address::from_raw(index), item)))
    // }
}

enum Entry<T> {
    Vacant { next_vacant: usize },
    Occupied(T),
}

impl<T> Entry<T> {
    fn get(&self) -> Option<&T> {
        match self {
            Self::Occupied(x) => Some(x),
            _ => None,
        }
    }

    fn get_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Occupied(x) => Some(x),
            _ => None,
        }
    }
}

pub struct Address<T>(usize, PhantomData<T>);

impl<T> Address<T> {
    pub fn from_raw(raw: usize) -> Self {
        Self(raw, PhantomData)
    }

    pub fn into_raw(self) -> usize {
        self.0
    }
}

impl<T> Debug for Address<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Address").field(&self.0).finish()
    }
}

impl<T> Clone for Address<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for Address<T> {}

impl<T> PartialEq for Address<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Address<T> {}

impl<T> Hash for Address<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
