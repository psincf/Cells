pub mod utils;

use crossbeam_channel::{Receiver, Sender};
use crossbeam_deque::{Stealer, Worker};
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::fence;
use std::sync::atomic::Ordering;
use std::sync::Barrier;
use std::thread::JoinHandle;

enum Work {
    Closure(ClosureBox),
    End,
}

type ClosureBox = Box<(dyn FnOnce() -> () + Send + Sync)>;

struct Channel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> Channel<T> {
    pub fn new() -> Channel<T> {
        let channel_raw = crossbeam_channel::unbounded();
        Channel {
            sender: channel_raw.0,
            receiver: channel_raw.1,
        }
    }
}

struct Deque<T> {
    sender: Worker<T>,
    receiver: Stealer<T>,
}

impl<T> Deque<T> {
    pub fn new() -> Deque<T> {
        let sender = crossbeam_deque::Worker::new_fifo();
        let receiver = sender.stealer();
        Deque {
            sender,
            receiver,
        }
    }
}

struct ThreadInfo {
    handle: JoinHandle<()>,
    index: Arc<AtomicUsize>,
}

pub struct ThreadPool {
    barrier: Arc<RwLock<Barrier>>,
    barrier_waiting: AtomicBool,
    threads: Vec<ThreadInfo>,
    threads_removed: Channel<usize>,
    channel: Deque<Work>,
}

impl ThreadPool {
    pub fn new(count: usize) -> ThreadPool {
        //core_affinity::set_for_current(core_affinity::CoreId{id: 0});
        let barrier = Arc::new(RwLock::new(Barrier::new(count as usize + 1)));
        let barrier_waiting = AtomicBool::new(false);
        let threads = Vec::new();
        let channel = Deque::new();
        let threads_removed = Channel::new();

        let mut threadpool = ThreadPool {
            barrier,
            barrier_waiting,
            threads,
            threads_removed,
            channel,
        };

        threadpool.set_threads(count);

        threadpool
    }

    pub fn num_threads(&self) -> usize {
        self.threads.len()
    }

    pub fn iter_threads(&self) -> std::ops::Range<usize> {
        0..self.num_threads()
    }

    fn add_thread(&mut self) {
        let receiver = self.channel.receiver.clone();
        let index = Arc::new(AtomicUsize::new(self.threads.len()));
        let index_clone = index.clone();
        let threads_removed = self.threads_removed.sender.clone();
        let handle = std::thread::spawn( move || {
            //core_affinity::set_for_current(core_affinity::CoreId{id: index_clone.load(Ordering::Relaxed) * 2 + 1});
            loop {
                if receiver.is_empty() {
                    std::hint::spin_loop();
                    std::thread::yield_now();
                    continue
                }
                let result = receiver.steal();
                
                let work = match result {
                    crossbeam_deque::Steal::Success(work_inner) => { Some(work_inner) }
                    _ => { None }
                };
                

                if work.is_some() {
                    let work = work.unwrap();
                    match work {
                        Work::Closure(closure) => { closure() },
                        Work::End => { break }
                    }
                } else {
                    std::hint::spin_loop();
                    std::thread::yield_now();
                }
            }
            threads_removed.send(index_clone.load(Ordering::SeqCst)).unwrap();
        });
        let thread_info = ThreadInfo {
            handle,
            index,
        };
        self.threads.push(thread_info);
    }

    fn remove_thread(&mut self) {
        self.channel.sender.push(Work::End);

        let thread_index = self.threads_removed.receiver.recv().unwrap();
        let thread_removed = self.threads.swap_remove(thread_index);
        thread_removed.handle.join().unwrap();
        if let Some(thread_info) = self.threads.get(thread_index) {
            thread_info.index.store(thread_index, Ordering::SeqCst);
        }
    }

    pub fn set_threads(&mut self, count: usize) {
        let actual_treads = self.threads.len();
        if count == actual_treads { return }
        
        self.unlock_barrier();
        if count > actual_treads {
            let threads_to_add = count - actual_treads;
            for _ in 0..threads_to_add {
                self.add_thread();
            }
        } else if count < actual_treads {
            let threads_to_remove = actual_treads - count;
            for _ in 0..threads_to_remove {
                self.remove_thread();
            }
        }
        self.update_barrier();
    }

    fn update_barrier(&mut self) {
        *self.barrier.write() = Barrier::new(self.threads.len() + 1);
    }

    pub fn lock_barrier(&self) {
        self.unlock_barrier();
        for _ in 0..self.threads.len() {
            let barrier = self.barrier.clone();
            unsafe { self.compute_unsafe( move || { barrier.read().wait(); } ) };
        }
        self.barrier_waiting.store(true, Ordering::SeqCst);
    }

    pub fn unlock_barrier(&self) {
        if self.barrier_waiting.load(Ordering::SeqCst) {
            self.barrier.read().wait();
            self.barrier_waiting.store(false, Ordering::SeqCst);
        }
    }

    pub fn sync_barrier(&self) {
        self.lock_barrier();
        self.unlock_barrier();
    }

    pub fn sync_spin(&self) {
        let busy_wait = Arc::new(AtomicUsize::new(0));
        let can_quit = Arc::new(AtomicBool::new(false));
        let num_threads = self.num_threads();
        for _ in 0..self.threads.len() {
            let busy_wait = busy_wait.clone();
            let can_quit = can_quit.clone();
            let function = Box::new( move || {
                busy_wait.fetch_add(1, Ordering::SeqCst);
                fence(Ordering::SeqCst);
                while !can_quit.load(Ordering::SeqCst) {
                    std::hint::spin_loop();
                    std::thread::yield_now(); 
                }
                fence(Ordering::SeqCst);
            });
            self.channel.sender.push(Work::Closure(function));
        }
        fence(Ordering::SeqCst);
        while busy_wait.load(Ordering::SeqCst) != num_threads { };
        can_quit.store(true, Ordering::SeqCst);
        fence(Ordering::SeqCst);
    }

    pub fn compute_static<F: FnOnce() -> () + Send + Sync + 'static>(&mut self, function: F) {
        unsafe { self.compute_unsafe(function) };
    }

    pub fn compute_static_each_thread<F: FnOnce() -> () + Clone + Send + Sync + 'static>(&mut self, function: F) {
        for _ in 0..self.num_threads().max(1) {
            self.compute_static(function.clone());
        }
    }

    pub fn compute_each_thread_and_wait<'a, F: FnOnce() -> () + Clone + Send + Sync + 'a>(&mut self, function: F) {
        for _ in 0..self.num_threads().max(1) {
            unsafe { self.compute_unsafe(function.clone()) };
        }
        self.sync_spin();
    }

    pub unsafe fn compute_unsafe<'a, F: FnOnce() -> () + Send + Sync + 'a>(&self, function: F) {
        let function = Box::new(function);
        let function = std::mem::transmute::<Box<dyn FnOnce() -> () + Send + Sync>, Box<dyn FnOnce() -> () + Send + Sync>>(function);
        if self.num_threads() == 0 { function(); return }
        self.channel.sender.push(Work::Closure(function));
    }

    pub unsafe fn compute_unsafe_boxed(&self, function: Box<dyn FnOnce() -> () + Send + Sync>) {
        let function = Box::new(function);
        let function = std::mem::transmute::<Box<dyn FnOnce() -> () + Send + Sync>, Box<dyn FnOnce() -> () + Send + Sync>>(function);
        if self.num_threads() == 0 { function(); return }
        self.channel.sender.push(Work::Closure(function));
    }

    pub unsafe fn compute_each_thread_unsafe<'a, F: FnOnce() -> () + Clone + Send + Sync + 'a>(&self, function: F) {
        for _ in 0..self.num_threads().max(1) {
            self.compute_unsafe(function.clone());
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.unlock_barrier();
        for _ in 0..self.threads.len() {
            self.channel.sender.push(Work::End);
        }
        let mut threads = Vec::new();
        std::mem::swap(&mut threads, &mut self.threads);
        for thread in threads {
            thread.handle.join().unwrap();
        }
    }
}