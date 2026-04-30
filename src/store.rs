//! A thin wrapper around a Vec and a newtype ID
//! to store a set of values.
pub struct Store<K, V> {
    items: Vec<(K, V)>,
    next_id: usize,
}

impl<K: PartialEq, V: PartialEq> PartialEq for Store<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for Store<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").field("items", &self.items).finish()
    }
}

trait KeyType: Copy + PartialOrd {
    fn from_usize(value: usize) -> Self;
}

impl<K, V> Default for Store<K, V> {
    fn default() -> Self {
        Store {
            items: Vec::new(),
            next_id: 0,
        }
    }
}

// Allow collecting into a store
impl<K: KeyType, V> FromIterator<V> for Store<K, V> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let mut store = Store::default();
        for item in iter {
            store.insert(item);
        }
        store
    }
}

impl<K: KeyType, V> Store<K, V> {
    pub fn insert(&mut self, value: V) -> K {
        let id = self.next_id + 1;
        let key = K::from_usize(id);
        self.items.push((key, value));
        self.next_id += 1;
        key
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.items
            .iter()
            .find_map(|(k, v)| (*k == key).then_some(v))
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.items
            .iter_mut()
            .find_map(|(k, v)| (*k == key).then_some(v))
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        if let Some(pos) = self.items.iter().position(|(k, _)| *k == key) {
            Some(self.items.remove(pos).1)
        } else {
            None
        }
    }
    pub fn retain(&mut self, mut f: impl FnMut(&V) -> bool) {
        self.items.retain(|(_, v)| f(v));
    }
    pub fn sort_by_field<T: Ord>(&mut self, mut f: impl FnMut(&V) -> T) {
        self.items.sort_by_key(|(_, v)| f(v));
    }
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.items.iter().map(|(k, v)| (*k, v))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.items.iter_mut().map(|(k, v)| (*k, v))
    }
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.items.iter().map(|(_, v)| v)
    }
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.items.iter_mut().map(|(_, v)| v)
    }
    pub fn keys(&self) -> impl Iterator<Item = K> {
        self.items.iter().map(|(k, _)| *k)
    }
    pub fn first(&self) -> Option<(K, &V)> {
        self.items.first().map(|(k, v)| (*k, v))
    }
    pub fn last(&self) -> Option<(K, &V)> {
        self.items.last().map(|(k, v)| (*k, v))
    }
    pub fn windows(&self, size: usize) -> impl Iterator<Item = Vec<(K, &V)>> {
        self.items
            .windows(size)
            .map(|window| window.iter().map(|(k, v)| (*k, v)).collect())
    }
}

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub struct $name(usize);

        impl KeyType for $name {
            fn from_usize(value: usize) -> Self {
                Self(value)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(WaypointId);
define_id!(EdgeId);
define_id!(WireLabelId);
define_id!(LabelId);
define_id!(RectId);
define_id!(RouteId);
