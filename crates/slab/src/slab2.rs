/*
pub struct Key<T> {
    index: usize, 
    marker: std::marker::PhantomData<*const T>,
}
*/

pub struct Data<T> {
    inner: Vec<T>,
    index_keys: Vec<usize>,
}

impl<T> Data<T> {
    pub fn new() -> Data<T> {
        Data {
            inner: Vec::new(),
            index_keys: Vec::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn push(&mut self, data: T, key: usize) {
        self.inner.push(data);
        self.index_keys.push(key);
    }
    pub fn swap_remove(&mut self, index: usize) {
        self.inner.swap_remove(index);
        self.index_keys.swap_remove(index);
    }

    pub fn swap_remove_and_test(&mut self, index: usize) -> Option<&usize> {
        self.swap_remove(index);
        self.index_keys.get(index)
    }

    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
        self.index_keys.shrink_to_fit();
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.index_keys.clear();
    }
}

pub type Key = usize;

pub struct Slab<T> {
    data: Data<T>,
    keys: Vec<Option<usize>>,
    free_keys: Vec<usize>,
}

impl<T> Slab<T> {
    pub fn new() -> Slab<T> {
        Slab {
            data: Data::new(),
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

        self.keys[next_key] = Some(self.data.len());

        self.data.push(data, next_key);

        /*
        Key {
            index: next_key,
            marker: std::marker::PhantomData,
        }
        */
        next_key
    }

    pub fn get(&self, key: Key) -> Option<&T> {
        let index_data = self.keys[key].unwrap();
        self.data.inner.get(index_data)
    }

    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        let index_data = self.keys[key].unwrap();
        self.data.inner.get_mut(index_data)
    }

    pub unsafe fn get_unchecked(&self, key: Key) -> &T {
        let index_data = self.keys[key].unwrap_unchecked();
        &self.data.inner.get_unchecked(index_data)
    }

    pub fn remove(&mut self, key: Key) {
        let index_data = self.keys[key].unwrap();
        if let Some(key_data_moved) = self.data.swap_remove_and_test(index_data) {
            *self.keys[*key_data_moved].as_mut().unwrap() = index_data;
        }
        self.keys[key] = None;
        self.free_keys.push(key);
    }

    pub unsafe fn remove_unchecked(&mut self, key: Key) {
        let index_data = self.keys[key].unwrap_unchecked();
        if let Some(key_data_moved) = self.data.swap_remove_and_test(index_data) {
            *self.keys[*key_data_moved].as_mut().unwrap() = index_data;
        }
        self.keys[key] = None;
        self.free_keys.push(key);
    }

    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
        for i in (0..self.keys.len()).rev() {
            if self.keys[i].is_none() {
                self.keys.remove(i);
            } else {
                break
            }
        }
        for i in (0..self.free_keys.len()).rev() {
            if self.free_keys[i] >= self.keys.len() {
                self.free_keys.swap_remove(i);
            }
        }
        self.free_keys.sort_unstable();
        self.free_keys.reverse();
        self.free_keys.shrink_to_fit();
        self.keys.shrink_to_fit();
    }

    pub fn inner_vec(&self) -> &Vec<T> {
        &self.data.inner
    }

    pub unsafe fn inner_vec_mut(&mut self) -> &mut Vec<T> {
        &mut self.data.inner
    }

    pub fn next_key_id(&self) -> usize {
        if let Some(index) = self.free_keys.last() {
            return *index
        } else {
            return self.keys.len()
        };
    }

    pub fn len(&self) -> usize {
        self.data.inner.len()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.keys.clear();
        self.free_keys.clear();
    }

    pub fn assert_correct(&self) {
        let mut num_key = 0;
        for key_option in self.keys.iter() {
            if let Some(key) = key_option {
                num_key += 1;
                if self.data.inner.get(*key).is_none() {
                    panic!("aaa");
                }
            }
        }
        assert!(num_key == self.data.inner.len());
        assert!(num_key == self.data.index_keys.len());
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.inner.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.inner.iter_mut()
    }
}

impl<T> Default for Slab<T> {
    fn default() -> Slab<T> {
        Slab::new()
    }
}

impl<T> std::ops::Index<Key> for Slab<T> {
    type Output = T;
    fn index(&self, index: Key) -> &T {
        &self.data.inner[self.keys[index].unwrap()]
    }
}

impl<T> std::ops::IndexMut<Key> for Slab<T> {
    fn index_mut(&mut self, index: Key) -> &mut T {
        &mut self.data.inner[self.keys[index].unwrap()]
    }
}


impl<T: Clone> Clone for Slab<T> {
    fn clone(&self) -> Slab<T> {
        Slab {
            data: Data {
                inner: self.data.inner.clone(),
                index_keys: self.data.index_keys.clone(),
            },
            keys: self.keys.clone(),
            free_keys: self.free_keys.clone(),
        }
    }
}