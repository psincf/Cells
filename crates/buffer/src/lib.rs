use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
//use parking_lot::{Mutex, MutexGuard};
use spin::{Mutex, MutexGuard};

pub struct Buffer<T> {
    size: AtomicUsize,
    inner: Mutex<VecDeque<T>>,
}

impl<T> Buffer<T> {
    pub fn new() -> Buffer<T> {
        Self::with_capacity(10)
    }

    pub fn with_capacity(capacity: usize) -> Buffer<T> {
        let size = AtomicUsize::new(0);
        let inner = Mutex::new(VecDeque::with_capacity(capacity));
        Buffer {
            size,
            inner,
        }
        
    }

    pub fn clear(&self) {
        self.size.store(0, Ordering::Relaxed);
        self.inner.lock().clear();
    }

    pub fn len(&self) -> usize {
        self.inner.lock().len()
    }

    pub fn send(&self, data: T) {
        self.size.fetch_add(1, Ordering::Relaxed);
        self.inner.lock().push_back(data);
    }

    pub fn is_some(&self) -> bool {
        self.size.load(Ordering::Relaxed) > 0
    }

    pub fn receive(&self) -> BufferIterator<T> {
        self.size.store(0, Ordering::Relaxed);
        let data = self.inner.lock();
        BufferIterator {
            data: data,
        }
    }
}

impl<T: Clone> Buffer<T> {
    pub fn receive_copy(&self) -> VecDeque<T> {
        let data = self.inner.lock().clone();
        return data
    }
}

pub struct BufferIterator<'a, T> {
    data: MutexGuard<'a, VecDeque<T>>,
}

impl<T> Iterator for BufferIterator<'_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.data.pop_front()
    }
}




//pub type BufferMulti<T> = BufferMulti<T>;

pub struct BufferMulti<T> {
    buffer_count: usize,
    size: AtomicUsize,
    inner: Vec<([u8; 128], Mutex<Vec<T>>)>, // TODO: do it better: array is for padding => no false sharing
}

impl<T> BufferMulti<T> {
    pub fn new() -> BufferMulti<T> {
        Self::with_capacity(10, 10)
    }

    pub fn with_capacity(capacity: usize, buffer_count: usize) -> BufferMulti<T> {
        let size = AtomicUsize::new(0);
        let mut inner = Vec::with_capacity(buffer_count);
        for _ in 0..buffer_count {
            inner.push(([0; 128], Mutex::new(Vec::with_capacity(capacity))));
        }
        BufferMulti {
            buffer_count,
            size,
            inner,
        }
        
    }

    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    pub fn send(&self, data: T) {
        let size = self.size.fetch_add(1, Ordering::Relaxed);
        self.inner[size % self.buffer_count].1.lock().push(data);
    }

    pub fn send_seed(&self, data: T, seed: usize) {
        self.size.fetch_add(1, Ordering::Relaxed); //TODO: change this
        self.inner[seed % self.buffer_count].1.lock().push(data);
    }

    pub fn is_some(&self) -> bool {
        self.size.load(Ordering::Relaxed) > 0
    }

    pub fn receive(&self) -> BufferMultiIterator<T> {
        self.size.store(0, Ordering::Relaxed);
        let mut data = Vec::new();
        for buffer in self.inner.iter() {
            data.append(&mut *buffer.1.lock());
        }
        //data.reverse(); // TODO: not necesary ?
        BufferMultiIterator {
            data: data,
        }
    }
}

pub struct BufferMultiIterator<T> {
    data: Vec<T>, //TODO: Ok ?
}

impl<T> Iterator for BufferMultiIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.data.pop()
    }
}