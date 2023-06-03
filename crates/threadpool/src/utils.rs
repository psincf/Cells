use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use super::ThreadPool;

pub use unsafe_ptr::UnsafePtr;

#[derive(Clone)]
pub struct AtomicRange {
    actual_index: Arc<AtomicUsize>,
    range: Range<usize>,
}

impl AtomicRange {
    pub fn new(range: Range<usize>) -> AtomicRange {
        AtomicRange {
            actual_index: Arc::new(AtomicUsize::new(range.start)),
            range: range,
        }
    }

    pub fn next_range(&self, step: usize) -> Option<Range<usize>> {
        let range_start = self.actual_index.fetch_add(step, Ordering::Relaxed);
        if range_start > self.range.end {
            return None;
        } else {
            let range_end = (range_start + step).min(self.range.end);
            if range_start == range_end {
                return None;
            }
            return Some(range_start..range_end);
        }
    }

    pub fn reset(&mut self, range: Range<usize>) {
        self.actual_index.store(range.start, Ordering::Relaxed);
        self.range = range;
    }
}

#[macro_export]
macro_rules! range_break {
    ($range: ident, $atomic_count: ident, $max: expr, $step: expr) => {
        let max = $max;
        let range_begin = $atomic_count.fetch_add($step, std::sync::atomic::Ordering::Relaxed);
        let range_max = (range_begin + $step).min(max);
        let $range = range_begin..range_max;

        if $range.start >= max {
            break
        }
    }
}

#[macro_export]
macro_rules! range_return {
    ($range: ident, $atomic_count: ident, $max: expr, $step: expr) => {
        let max = $max;
        let range_begin = $atomic_count.fetch_add($step, std::sync::atomic::Ordering::Relaxed);
        let range_max = (range_begin + $step).min(max);
        let $range = range_begin..range_max;

        if $range.start >= max {
            return
        }
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! compute_range_each_thread_join {
        ($threadpool: expr, $range: expr, $step: expr, $function: expr) => {
            let atomic_range = $crate::utils::AtomicRange::new($range);
            for _ in 0..$threadpool.num_threads().max(1) {
                let atomic_range = atomic_range.clone();
                let function_final = move || {
                    loop {
                        let range = atomic_range.next_range($step);
                        if range.is_none() { break }
                        let range = range.unwrap();
                        for i in range {
                            $function(i); //TODO function may be not inlined!!
                        }
                    }
                };
                unsafe { $threadpool.compute_unsafe(function_final) };
            }
            $threadpool.sync_spin();
        }
    }

    #[macro_export]
    macro_rules! compute_iter_each_thread_join {
        ($threadpool: expr, $collection: expr, $step: expr, $function: expr) => {
            $crate::compute_range_each_thread_join!($threadpool, 0..$collection.len(), $step, |i| { let entity = unsafe { $collection.get_unchecked(i) }; $function(entity) });
        }
    }

    #[macro_export]
    macro_rules! compute_iter_mut_each_thread_join {
        ($threadpool: expr, $collection: expr, $step: expr, $function: expr) => {
            let collection_send = $crate::utils::UnsafePtr::new($collection);
            $crate::compute_range_each_thread_join!($threadpool, 0..$collection.len(), $step, |i| { let entity = unsafe { (collection_send.ref_mut()).get_unchecked_mut(i) }; $function(entity) });
        }
    }
}

pub trait ParallelIterator {
    fn compute_range_each_thread_join<F: Fn(usize) -> () + Clone + Send + Sync>(
        &self,
        range: Range<usize>,
        step: usize,
        function: F
    );
    fn compute_iter_each_thread_join<F: Fn(&T) -> () + Clone + Send + Sync, T: Send + Sync>(
        &self,
        array: &[T],
        step: usize,
        function: F
    );
    fn compute_iter_mut_each_thread_join<F: Fn(&mut T) -> () + Clone + Send + Sync, T: Send + Sync>(
        &self,
        array: &mut [T],
        step: usize,
        function: F
    );
}

impl ParallelIterator for ThreadPool {
    fn compute_range_each_thread_join<F: Fn(usize) -> () + Clone + Send + Sync>(
        &self,
        range: Range<usize>,
        step: usize,
        function: F
    ) {
        let atomic_range = AtomicRange::new(range);
        for _ in 0..self.num_threads().max(1) {
            let atomic_range = atomic_range.clone();
            let function = function.clone();
            let function_final = move || {
                loop {
                    let range = atomic_range.next_range(step);
                    if range.is_none() { break }
                    let range = range.unwrap();
                    for i in range {
                        function(i); //TODO function may be not inlined!!
                    }
                }
            };
            unsafe { self.compute_unsafe(function_final) };
        }
        self.sync_spin();
    }
    
    fn compute_iter_each_thread_join<F: Fn(&T) -> () + Clone + Send + Sync, T: Send + Sync>(
        &self,
        array: &[T],
        step: usize,
        function: F
    ) {
        ParallelIterator::compute_range_each_thread_join(self, 0..array.len(), step, |i| {
            let entity = unsafe { array.get_unchecked(i) };
            function(entity);
        });
    }

    fn compute_iter_mut_each_thread_join<F: Fn(&mut T) -> () + Clone + Send + Sync, T: Send + Sync>(
        &self,
        array: &mut [T],
        step: usize,
        function: F
    ) {
        let array_send = unsafe_ptr::UnsafePtr::new(array as *mut [T]);
        ParallelIterator::compute_range_each_thread_join(self, 0..array.len(), step, move |i| {
            let entity = unsafe { (array_send.ref_mut()).get_unchecked_mut(i) };
            function(entity);
        });
    }
    /*
    fn compute_iter_each_thread_join<F: Fn(&T) -> () + Clone + Send + Sync, T: Send + Sync>(
        &self,
        array: &[T],
        step: usize,
        function: F
    ) {
        let atomic_range = AtomicRange::new(0..array.len());
        for _ in 0..self.num_threads().max(1) {
            let atomic_range = atomic_range.clone();
            let function = function.clone();
            let function_final = move || {
                loop {
                    let range = atomic_range.next_range(step);
                    if range.is_none() { break }
                    let range = range.unwrap();
                    for i in range {
                        let entity = unsafe { array.get_unchecked(i) };
                        //let entity = &mut array[i];
                        function(entity); //TODO function may be not inlined!!
                    }
                }
            };
            unsafe { self.compute_unsafe(function_final) };
        }
        self.sync_spin();
    }

    fn compute_iter_mut_each_thread_join<F: Fn(&mut T) -> () + Clone + Send + Sync, T: Send + Sync>(
        &self,
        array: &mut [T],
        step: usize,
        function: F
    ) {
        let atomic_range = AtomicRange::new(0..array.len());
        for _ in 0..self.num_threads().max(1) {
            let atomic_range = atomic_range.clone();
            let array = &mut *array;
            let function = function.clone();
            let function_final = move || {
                loop {
                    let range = atomic_range.next_range(step);
                    if range.is_none() { break }
                    let range = range.unwrap();
                    for i in range {
                        let entity = unsafe { array.get_unchecked_mut(i) };
                        //let entity = &mut array[i];
                        function(entity); //TODO function may be not inlined!!
                    }
                }
            };
            unsafe { self.compute_unsafe(function_final) };
        }
        self.sync_spin();
    }
    */
}


pub trait SliceLike<T: Send + Sync>: std::ops::Index<usize, Output = T> + Send + Sync {
    unsafe fn _get_unchecked(&self, index: usize) -> &T;
    unsafe fn _get_unchecked_mut(&mut self, index: usize) -> &mut T;
    fn _len(&self) -> usize;
}

macro_rules! impl_slice_like_trait {
    ($MyStruct: ty) => {
        impl<T: Send + Sync> SliceLike<T> for $MyStruct {
            unsafe fn _get_unchecked(&self, index: usize) -> &T {
                self.get_unchecked(index)
            }
            unsafe fn _get_unchecked_mut(&mut self, index: usize) -> &mut T {
                self.get_unchecked_mut(index)
            }
            fn _len(&self) -> usize {
                self.len()
            }
        }
    };
}

impl_slice_like_trait!(Vec<T>);
impl_slice_like_trait!([T]);

pub trait ParallelIteratorGeneric {
    fn compute_range_each_thread_join<F: Fn(usize) -> () + Clone + Send + Sync>(
        &self,
        range: Range<usize>,
        step: usize,
        function: F
    );
    fn compute_iter_each_thread_join<F: Fn(&T) -> () + Clone + Send + Sync, T: Send + Sync>(
        &self,
        array: &dyn SliceLike<T>,
        step: usize,
        function: F
    );
    fn compute_iter_mut_each_thread_join<F: Fn(&mut T) -> () + Clone + Send + Sync, T: Send + Sync>(
        &self,
        array: &mut dyn SliceLike<T>,
        step: usize,
        function: F
    );
}

impl ParallelIteratorGeneric for ThreadPool {
    fn compute_range_each_thread_join<F: Fn(usize) -> () + Clone + Send + Sync>(
        &self,
        range: Range<usize>,
        step: usize,
        function: F
    ) {
        let atomic_range = AtomicRange::new(range);
        for _ in 0..self.num_threads().max(1) {
            let atomic_range = atomic_range.clone();
            let function = function.clone();
            let function_final = move || {
                loop {
                    let range = atomic_range.next_range(step);
                    if range.is_none() { break }
                    let range = range.unwrap();
                    for i in range {
                        function(i); //TODO function may be not inlined!!
                    }
                }
            };
            unsafe { self.compute_unsafe(function_final) };
        }
        self.sync_spin();
    }

    fn compute_iter_each_thread_join<F: Fn(&T) -> () + Clone + Send + Sync, T: Send + Sync>(
        &self,
        array: &dyn SliceLike<T>,
        step: usize,
        function: F
    ) {
        let atomic_range = AtomicRange::new(0..array._len());
        for _ in 0..self.num_threads().max(1) {
            let atomic_range = atomic_range.clone();
            let function = function.clone();
            let function_final = move || {
                loop {
                    let range = atomic_range.next_range(step);
                    if range.is_none() { break }
                    let range = range.unwrap();
                    for i in range {
                        let entity = unsafe { array._get_unchecked(i) };
                        //let entity = &mut array[i];
                        function(entity); //TODO function may be not inlined!!
                    }
                }
            };
            unsafe { self.compute_unsafe(function_final) };
        }
        self.sync_spin();
    }

    fn compute_iter_mut_each_thread_join<F: Fn(&mut T) -> () + Clone + Send + Sync, T: Send + Sync>(
        &self,
        array: &mut dyn SliceLike<T>,
        step: usize,
        function: F
    ) {
        let atomic_range = AtomicRange::new(0..array._len());
        for _ in 0..self.num_threads().max(1) {
            let atomic_range = atomic_range.clone();
            let array = &mut *array;
            let function = function.clone();
            let function_final = move || {
                loop {
                    let range = atomic_range.next_range(step);
                    if range.is_none() { break }
                    let range = range.unwrap();
                    for i in range {
                        let entity = unsafe { array._get_unchecked_mut(i) };
                        //let entity = &mut array[i];
                        function(entity); //TODO function may be not inlined!!
                    }
                }
            };
            unsafe { self.compute_unsafe(function_final) };
        }
        self.sync_spin();
    }
}