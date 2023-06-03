use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::thread;
use std::sync::Barrier;

struct Synchronization {
    pub threads: Vec<JoinHandle<()>>,
    pub beginning_barrier: Arc<Barrier>,
    pub game_running: Arc<AtomicBool>,
}

impl Synchronization {
    fn new() -> Synchronization {
        Synchronization {
            threads: Vec::new(),
            beginning_barrier: Arc::new(Barrier::new(0)),
            game_running: Arc::new(AtomicBool::new(true)),
        }
    }

    fn add_thread(&mut self, handle: JoinHandle<()>) {
        self.threads.push(handle);
    }

    fn clone_barrier_and_running(&self) -> (Arc<Barrier>, Arc<AtomicBool>) {
        (self.beginning_barrier.clone(), self.game_running.clone())
    }

    fn reset(&mut self) {
        self.game_running.store(false, Ordering::Relaxed);
        while let Some(thread) = self.threads.pop() {
            let _ = thread.join();
        }
        assert!(self.threads.is_empty());
    }
}

pub struct AppRunnerInfos {
    synchronization: Synchronization,
    functions: Vec<Box<dyn Fn() + Send + 'static>>,
}

impl AppRunnerInfos {
    pub fn new() -> AppRunnerInfos {
        AppRunnerInfos {
            synchronization: Synchronization::new(),
            functions: Vec::new(),
        }
    }

    pub fn add_function<F: Fn() + Send + 'static>(&mut self, function: F) {
        self.functions.push(Box::new(function));
    }

    pub fn add_function_boxed(&mut self, function: Box<dyn Fn() + Send + 'static>) {
        self.functions.push(function);
    }

    pub fn add_functions<F: Fn() + Send + 'static>(&mut self, mut functions: Vec<F>) {
        while let Some(f) = functions.pop() {
            self.add_function_boxed(Box::new(f));
        }
    }

    pub fn running(&self) -> bool {
        self.synchronization.game_running.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.synchronization.game_running.store(false, Ordering::Relaxed);
    }

    pub fn stop_and_wait(&mut self) {
        self.stop();
        self.synchronization.reset();
    }

    pub fn run_singlethread(&mut self) {
        for function in self.functions.iter() {
            if self.synchronization.game_running.load(Ordering::Relaxed) {
                function();
            }
        }
    }

    pub fn run_multithread(&mut self) {
        self.synchronization.beginning_barrier = Arc::new(Barrier::new(self.functions.len() + 1));
        for function in self.functions.drain(..) {
            let (beginning_barrier, game_running) = self.synchronization.clone_barrier_and_running();
            let handle = thread::spawn(move || {
                beginning_barrier.wait();
                while game_running.load(Ordering::Relaxed) {
                    function();
                }
            });
            self.synchronization.add_thread(handle);
        }
        
        self.synchronization.beginning_barrier.wait();
    }
}