pub mod slab2;
pub use slab2::*;

//TODO: Slab Trait ?
//TODO: Better organization

/*
use unwrap_unchecked::*;

pub struct DataSlot<T> { //TODO: Separate inner and index_key in ifferents vector ?
    inner: T,
    index_key: usize,
}

/*
pub struct Key<T> {
    index: usize, 
    marker: std::marker::PhantomData<*const T>,
}
*/

pub type Key = usize;

pub struct Slab<T> {
    inner: Vec<DataSlot<T>>,
    keys: Vec<Option<usize>>,
    free_keys: Vec<usize>,
}

impl<T> Slab<T> {
    pub fn new() -> Slab<T> {
        Slab {
            inner: Vec::new(),
            keys: Vec::new(),
            free_keys: Vec::new(),
        }
    }

    pub fn insert(&mut self, data: T) -> Key {
        let next_key = if let Some(index) = self.free_keys.pop() {
            index
        } else {
            self.keys.push(None);
            self.keys.len() - 1
        };

        self.keys[next_key] = Some(self.inner.len());

        self.inner.push(
            DataSlot {
                inner: data,
                index_key: next_key,
            }
        );

        /*
        Key {
            index: next_key,
            marker: std::marker::PhantomData,
        }
        */
        next_key
    }

    pub fn get(&self, key: Key) -> &T {
        let index_data = self.keys[key].unwrap();
        &self.inner[index_data].inner
    }

    pub unsafe fn get_unchecked(&self, key: Key) -> &T {
        let index_data = self.keys[key].unwrap_unchecked();
        &self.inner.get_unchecked(index_data).inner
    }

    pub fn remove(&mut self, key: Key) {
        let index_data = self.keys[key].unwrap();
        self.inner.swap_remove(index_data);
        if let Some(data_slot) = self.inner.get(index_data) {
            let key_data_moved = data_slot.index_key;
            *self.keys[key_data_moved].as_mut().unwrap() = index_data;
        }
        self.keys[key] = None;
        self.free_keys.push(key);
    }

    pub unsafe fn remove_unchecked(&mut self, key: Key) {
        let index_data = self.keys[key].unwrap_unchecked();
        self.inner.swap_remove(index_data);
        if let Some(data_slot) = self.inner.get(index_data) {
            let key_data_moved = data_slot.index_key;
            *self.keys[key_data_moved].as_mut().unwrap_unchecked() = index_data;
        }
        self.keys[key] = None;
        self.free_keys.push(key);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.keys.clear();
        self.free_keys.clear();
    }

    pub fn assert_correct(&self) {
        let mut num_key = 0;
        for key_option in self.keys.iter() {
            if let Some(key) = key_option {
                num_key += 1;
                if self.inner.get(*key).is_none() {
                    panic!("aaa");
                }
            }
        }
        assert!(num_key == self.inner.len());
    }
}

impl<T> Default for Slab<T> {
    fn default() -> Slab<T> {
        Slab::new()
    }
}


impl<T> Slab<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            index: 0,
            data: &self.inner
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            index: 0,
            data: &mut self.inner
        }
    }
}

pub struct Iter<'a, T> {
    index: usize,
    data: &'a Vec<DataSlot<T>>,
}

impl<'a, T> std::iter::Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.index == self.data.len() { return None }
        else  {
            let data = unsafe { &self.data.get_unchecked(self.index).inner };
            self.index += 1;
            return Some(data);
        }
    }
}

pub struct IterMut<'a, T> {
    index: usize,
    data: &'a mut Vec<DataSlot<T>>,
}

impl<'a, T> std::iter::Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        if self.index == self.data.len() { return None }
        else  {
            let data = unsafe { &mut self.data.get_unchecked_mut(self.index).inner };
            let data = unsafe { std::mem::transmute::<&mut T, &mut T>(data) };
            self.index += 1;
            return Some(data);
        }
    }
}

impl<T> std::ops::Index<Key> for Slab<T> {
    type Output = T;
    fn index(&self, index: Key) -> &T {
        &self.inner[self.keys[index].unwrap()].inner
    }
}

impl<T> std::ops::IndexMut<Key> for Slab<T> {
    fn index_mut(&mut self, index: Key) -> &mut T {
        &mut self.inner[self.keys[index].unwrap()].inner
    }
}
*/