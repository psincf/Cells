#[allow(dead_code)]
pub struct VecChunk<T> {
    size: usize,
    inner: Vec<Vec<T>>,
}

#[allow(dead_code)]
impl<T> VecChunk<T> {
    pub fn new() -> VecChunk<T> {
        VecChunk::with_capacity(10_000)
    }

    fn new_chunk(&mut self) {
        self.inner.push(Vec::with_capacity(self.size));
    }

    pub fn with_capacity(capacity: usize) -> VecChunk<T> {
        let mut inner = Vec::with_capacity(10);
        inner.push(Vec::with_capacity(capacity));

        VecChunk {
            size: capacity,
            inner,
        }
    }

    pub fn clear(&mut self) {
        for index in 0..self.inner.len() {
            self.inner[index].clear();
        }
        self.inner.truncate(1);
    }

    pub fn as_vec_of_vec(&mut self) -> &Vec<Vec<T>> {
        let vec = &self.inner;
        return vec;
    }
    pub fn push(&mut self, data: T) {
        if self.inner.last().unwrap().len() == self.size {
            self.new_chunk();
        }
        self.inner.last_mut().unwrap().push(data);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let x = index / self.size;
        let y = index % self.size;
        self.inner.get(x).map_or(None, |vec| vec.get(y))
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let x = index / self.size;
        let y = index % self.size;
        self.inner.get_mut(x).map_or(None, |vec| vec.get_mut(y))
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        let x = index / self.size;
        let y = index % self.size;
        self.inner.get_unchecked(x).get_unchecked(y)
    }

    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        let x = index / self.size;
        let y = index % self.size;
        self.inner.get_unchecked_mut(x).get_unchecked_mut(y)
    }

    pub fn len(&self) -> usize {
        (self.inner.len() - 1) * self.size + self.inner.last().unwrap().len()
    }
    
    pub unsafe fn set_len(&mut self, new_len: usize) {
        let chunk_count = (new_len / self.size) + 1;
        let last_chunk_len = new_len % self.size;

        for index in 0..chunk_count {
            self.inner[index].set_len(self.size);
            if index == chunk_count - 1 { self.inner[index].set_len(last_chunk_len); }
        }
    }

    pub fn reserve(&mut self, new_capacity: usize) {
        self.inner[0].reserve(self.size); // Not good, but necessary!!
        
        let chunks_total_to_allocate = new_capacity / self.size + 1;
        if chunks_total_to_allocate <= self.inner.len() { return }

        let chunks_to_allocate = chunks_total_to_allocate - self.inner.len();
        for _ in 0..chunks_to_allocate {
            self.new_chunk();
        }
    }

    pub fn swap_remove(&mut self, index: usize) {
        let x = index / self.size;
        let y = index % self.size;
        
        let last = self.inner.last_mut().unwrap().pop().unwrap();

        if let Some(to_replace) = self.inner[x].get_mut(y) {
            *to_replace = last;
        }

        if self.inner.last().unwrap().len() == 0 {
            self.inner.pop();
        }
    }

    pub fn iter(&self) -> VecChunkIter<T> {
        VecChunkIter {
            size: self.size,
            inner: &self.inner,
            index: 0,
        }
    }
    
    pub fn iter_mut(&mut self) -> VecChunkIterMut<T> {
        VecChunkIterMut {
            size: self.size,
            inner: &mut self.inner,
            index: 0,
        }
    }
}

impl<T> std::ops::Index<usize> for VecChunk<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        let x = index / self.size;
        let y = index % self.size;
        &self.inner[x][y]
    }
}

impl<T> std::ops::IndexMut<usize> for VecChunk<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        let x = index / self.size;
        let y = index % self.size;
        &mut self.inner[x][y]
    }
}

pub struct VecChunkIter<'a, T> {
    size: usize,
    inner: &'a Vec<Vec<T>>,
    index: usize,
}

pub struct VecChunkIterMut<'a, T> {
    size: usize,
    inner: &'a mut Vec<Vec<T>>,
    index: usize,
}

impl<'a, T> Iterator for VecChunkIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let x = self.index / self.size;
        let y = self.index % self.size;
        self.index += 1;
        self.inner.get(x).and_then(|vec| vec.get(y))
    }
}

impl<'a, T> Iterator for VecChunkIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        let inner = unsafe { &mut *(self.inner as *mut Vec<Vec<T>>) };
        let x = self.index / self.size;
        let y = self.index % self.size;
        self.index += 1;
        if inner.get_mut(x).is_some() {
            let result = inner[x].get_mut(y);
            return result
        }
        return None
    }
}

impl<T: Send + Sync> threadpool::utils::SliceLike<T> for VecChunk<T> {
    unsafe fn _get_unchecked(&self, index: usize) -> &T { self.get_unchecked(index) }
    unsafe fn _get_unchecked_mut(&mut self, index: usize) -> &mut T { self.get_unchecked_mut(index) }
    fn _len(&self) -> usize { self.len() }
}

impl<T: Clone> Clone for VecChunk<T> {
    fn clone(&self) -> Self {
        Self {
            size: self.size,
            inner: self.inner.clone()
        }
    }
}