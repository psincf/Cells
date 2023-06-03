//pub mod vec_bool;
pub mod vec_chunk;
pub mod message_box;

pub fn i32_mul_f32_2(number: i32, mul: f32) -> i32 {
    let temp = number * mul.floor() as i32;

    let mul_fract = mul.fract();
    let temp_fract = (number / 10_000) * (mul_fract * 10_000.0).floor() as i32 
    + (number % 10_000) * ((mul_fract * 10_000.0).floor() as i32) / 10_000;
    
    let mul_fract_2 = (mul.fract() * 10_000.0).fract();
    let ratio_div = 10_000_000;
    let ratio_div_2 = 1_000;
    let temp_fract_2 = (number / ratio_div) * (mul_fract_2 * 1_000.0).floor() as i32 
    + ((number % ratio_div) / ratio_div_2) * ((mul_fract_2 * 1_000.0).floor() as i32) / (ratio_div / ratio_div_2);

    return temp + temp_fract + temp_fract_2;
}

#[macro_export]
macro_rules! new_timer_monothread_generic {
    ($benchmark: expr, $timer: ident, $name: expr) => {
        #[cfg(not(feature="shipping"))]
        let $timer = $benchmark.new_timer($name);
        #[cfg(feature="shipping")]
        let $timer = 0i32;
    };
}


#[derive(Clone, Copy)]
pub enum BufferChoice {
    First,
    Second,
}


//use std::sync::atomic::{AtomicUsize, Ordering};
//use parking_lot::{Mutex, MutexGuard};



/*
pub struct DoubleBuffer<T> {
    buffer1: Buffer<T>,
    buffer2: Buffer<T>,
}

impl<T> DoubleBuffer<T> {
    pub fn new() -> DoubleBuffer<T> {
        Self::with_capacity(10)
    }

    pub fn with_capacity(capacity: usize) -> DoubleBuffer<T> {
        let size = AtomicUsize::new(0);
        let inner = Mutex::new(Vec::with_capacity(capacity));
        let buffer1 = Buffer {
            size,
            inner,
        };
        let size = AtomicUsize::new(0);
        let inner = Mutex::new(Vec::with_capacity(capacity));
        let buffer2 = Buffer {
            size,
            inner,
        };

        DoubleBuffer {
            buffer1,
            buffer2,
        }        
    }

    pub fn send(&self, data: T, buffer: BufferChoice) {
        match buffer {
            BufferChoice::First => {
                self.buffer1.send(data);
            },

            BufferChoice::Second => {
                self.buffer2.send(data);
            },
        }
    }

    pub fn is_some(&self, buffer: BufferChoice) -> bool {
        match buffer {
            BufferChoice::First => {
                self.buffer1.is_some()
            },

            BufferChoice::Second => {
                self.buffer2.is_some()
            },
        }
    }

    pub fn receive(&self, buffer: BufferChoice) -> BufferIterator<T> {
        match buffer {
            BufferChoice::First => {
                self.buffer1.receive()
            },

            BufferChoice::Second => {
                self.buffer2.receive()
            },
        }
    }
}
*/

#[allow(dead_code)]
//#[derive(Clone)]
pub struct VecSpecial<T> {
    size: usize,
    inner: Vec<Vec<T>>,
    len: (usize, usize),
    chunk_allocated_count: usize,
}

#[allow(dead_code)]
impl<T> VecSpecial<T> {
    pub fn new() -> VecSpecial<T> {
        VecSpecial::with_capacity(10_000)
    }

    fn new_chunk(&mut self) {
        self.inner.push(Vec::with_capacity(self.size));
        self.chunk_allocated_count += 1;
    }

    pub fn with_capacity(capacity: usize) -> VecSpecial<T> {
        let mut inner = Vec::with_capacity(10);
        inner.push(Vec::with_capacity(capacity));

        VecSpecial {
            size: capacity,
            inner,
            len: (1, 0),
            chunk_allocated_count: 1,
        }
    }

    pub fn clear(&mut self) {
        for index in 0..self.len.0 {
            self.inner[index].clear();
        }
        self.len = (1,0);
    }

    pub fn to_vec(&mut self) -> &Vec<Vec<T>> {
        let vec = &mut self.inner;
        return vec;
    }

    pub fn push(&mut self, data: T) {
        if self.len.1 == self.size {
            self.len.0 += 1;
            self.len.1 = 0;
            if self.len.0 > self.chunk_allocated_count { 
                self.new_chunk();
            }
        }
        self.len.1 += 1;
        self.inner[self.len.0 - 1].push(data);
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
        (self.len.0 - 1) * self.size + self.len.1
    }

    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.len.0 = (new_len / self.size) + 1;
        self.len.1 = new_len % self.size;

        for index in 0..self.len.0 {
            self.inner[index].set_len(self.size);
            if index == self.len.0 - 1 { self.inner[index].set_len(new_len % self.size); }
        }
    }

    pub fn reserve(&mut self, new_capacity: usize) {
        self.inner[0].reserve(self.size); // Not good, but necessary!!
        
        let chunks_total_to_allocate = new_capacity / self.size + 1;
        if chunks_total_to_allocate <= self.chunk_allocated_count { return }

        let chunks_to_allocate = chunks_total_to_allocate - self.chunk_allocated_count;
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

        self.len.1 -= 1;

        if self.len.1 == 0 {
            self.len.0 -= 1;
            self.len.1 = self.size;
            self.inner.pop();
        }
    }

    pub fn iter(&self) -> VecSpecialIter<T> {
        VecSpecialIter {
            size: self.size,
            inner: &self.inner,
            index: 0,
        }
    }
    
    pub fn iter_mut(&mut self) -> VecSpecialIterMut<T> {
        VecSpecialIterMut {
            size: self.size,
            inner: &mut self.inner,
            index: 0,
        }
    }
}

impl<T> std::ops::Index<usize> for VecSpecial<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        let x = index / self.size;
        let y = index % self.size;
        &self.inner[x][y]
    }
}

impl<T> std::ops::IndexMut<usize> for VecSpecial<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        let x = index / self.size;
        let y = index % self.size;
        &mut self.inner[x][y]
    }
}

pub struct VecSpecialIter<'a, T> {
    size: usize,
    inner: &'a Vec<Vec<T>>,
    index: usize,
}

pub struct VecSpecialIterMut<'a, T> {
    size: usize,
    inner: &'a mut Vec<Vec<T>>,
    index: usize,
}

impl<'a, T> Iterator for VecSpecialIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let x = self.index / self.size;
        let y = self.index % self.size;
        self.index += 1;
        self.inner.get(x).and_then(|vec| vec.get(y))
    }
}

impl<'a, T> Iterator for VecSpecialIterMut<'a, T> {
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

impl<T: Send + Sync> threadpool::utils::SliceLike<T> for VecSpecial<T> {
    unsafe fn _get_unchecked(&self, index: usize) -> &T { self.get_unchecked(index) }
    unsafe fn _get_unchecked_mut(&mut self, index: usize) -> &mut T { self.get_unchecked_mut(index) }
    fn _len(&self) -> usize { self.len() }
}

/*
pub struct VecBox<T> {
    inner: Vec<Box<T>>,
}

impl<T> VecBox<T> {
    pub fn new() -> VecBox<T> {
        let inner = Vec::new();
        VecBox {
            inner,
        }
    }

    pub fn push(&mut self, data: T) {
        self.inner.push(Box::new(data));
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> VecBoxIter<T> {
        VecBoxIter {
            inner: &self.inner,
            index: 0,
        }
    }
    
    pub fn iter_mut(&mut self) -> VecBoxIterMut<T> {
        VecBoxIterMut {
            inner: &mut self.inner,
            index: 0,
        }
    }

    pub fn swap_remove(&mut self, index: usize) {
        self.inner.swap_remove(index);
    }
}

impl<T> std::ops::Index<usize> for VecBox<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.inner[index]
    }
}

impl<T> std::ops::IndexMut<usize> for VecBox<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {        
        &mut self.inner[index]
    }
}

pub struct VecBoxIter<'a, T> {
    inner: &'a Vec<Box<T>>,
    index: usize,
}

pub struct VecBoxIterMut<'a, T> {
    inner: &'a mut Vec<Box<T>>,
    index: usize,
}

impl<'a, T> Iterator for VecBoxIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let index = self.index;
        self.index += 1;
        self.inner.get(index).and_then(|boxed| Some(&**boxed) )
    }
}

impl<'a, T> Iterator for VecBoxIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'static mut T> {
        let inner = unsafe { &mut *(self.inner as *mut Vec<Box<T>>) };
        let index = self.index;
        self.index += 1;
        inner.get_mut(index).and_then(|boxed| Some(&mut **boxed) )
    }
}
*/

pub struct VecUniqueIndex {
    inner: usize,
    free: Vec<usize>,
}

impl VecUniqueIndex {
    pub fn new() -> VecUniqueIndex {
        VecUniqueIndex::with_capacity(10)
    }

    pub fn with_capacity(capacity: usize) -> VecUniqueIndex {
        VecUniqueIndex {
            inner: 0,
            free: Vec::with_capacity(capacity),
        }
    }

    /*
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }
    */

    pub fn gen_id(&mut self) -> usize {
        if self.free.len() > 0 {
            self.free.remove(self.free.len() - 1)
        } else {
            let id = self.inner;
            self.inner += 1;
            id
        }
    }

    pub fn remove(&mut self, id: usize) {
        self.free.push(id);
    }
}