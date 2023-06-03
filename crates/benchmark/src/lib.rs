mod accumulator;
use accumulator::Accumulator;

pub mod timer_node;

use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

//TODO: Add multithreading

struct BenchmarkInner {
    count: AtomicUsize,
    level: AtomicUsize,
    data: Mutex<HashMap<String, TimerInfo>>,
}

impl BenchmarkInner {
    pub fn new() -> BenchmarkInner {
        BenchmarkInner {
            count: AtomicUsize::new(0),
            level: AtomicUsize::new(0),
            data: Mutex::new(HashMap::new()),
        }
    }
}

pub struct Benchmark {
    inner: Arc<BenchmarkInner>,
    stored: Mutex<Vec<(String, TimerInfo)>>,
    accumulator: Accumulator,
}

impl Benchmark {
    pub fn new(ratio_average: usize) -> Benchmark {
        Benchmark {
            inner: Arc::new(BenchmarkInner::new()),
            stored: Mutex::new(Vec::new()),
            accumulator: Accumulator::new(ratio_average),
        }
    }

    pub fn new_timer(&self, name: &str) -> Option<Timer> {
        #[cfg(not(feature = "shipping"))]
        return Some(Timer::new(self.inner.clone(), name));
    }

    pub fn save(&self) {
        let mut data = Vec::new();
        for timer in self.inner.data.lock().unwrap().clone().into_iter() {
            data.push(timer);
        }

        *self.stored.lock().unwrap() = data;

        self.accumulator.accumulate(&self.inner.data);
    }

    pub fn get(&self) -> Vec<(String, TimerInfo)> {
        let data = self.stored.lock().unwrap().clone();
        data
    }

    pub fn get_in_order(&self) -> Vec<(String, TimerInfo)> {
        let mut data = self.get();
        data.sort_unstable_by(| a, b | { a.1.count.cmp(&b.1.count) } );
        data
    }

    pub fn get_average(&self) -> Vec<(String, TimerInfo)> {
        self.accumulator.get()
    }

    pub fn get_average_in_order(&self) -> Vec<(String, TimerInfo)> {
        let mut data = self.get_average();
        data.sort_unstable_by(| a, b | { a.1.count.cmp(&b.1.count) } );
        data
    }

    pub fn clear(&self) {
        self.inner.data.lock().unwrap().clear();
        self.inner.count.store(0, Ordering::Relaxed);
        self.inner.level.store(0, Ordering::Relaxed);
    }

    pub fn clear_accumulator(&self) {
        self.accumulator.clear();
    }
}

pub struct Timer {
    benchmark: Arc<BenchmarkInner>,
    local_level: usize,
    count: usize,
    name: String,
    instant: std::time::Instant,
}

impl Timer {
    fn new(benchmark: Arc<BenchmarkInner>, name: &str) -> Timer {
        let name = name.to_owned();
        let count = benchmark.count.fetch_add(1, Ordering::Relaxed);
        let local_level = benchmark.level.fetch_add(1, Ordering::Relaxed);
        Timer {
            benchmark,
            local_level,
            count,
            name,
            instant: std::time::Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let _replace = self.benchmark.data.lock().unwrap().insert(
            self.name.clone(),
            TimerInfo {
                duration: self.instant.elapsed(),
                level: self.local_level,
                count: self.count,
            }
        );
        //assert!(replace.is_none(), "{}", self.name.clone());

        self.benchmark.level.fetch_sub(1, Ordering::Relaxed);
    }
}

#[derive(Clone, Debug)]
pub struct TimerInfo {
    pub duration: Duration,
    pub level: usize,
    pub count: usize,
}