use crate::TimerInfo;

use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[derive(Clone, Debug)]
pub struct TimerNode {
    timer: TimerInfo,
    nodes: HashMap<String, TimerNode>,
}

impl TimerNode {
    fn get_simple(&self, name: &String, vec: &mut Vec<(String, TimerInfo)>) {
        for (name, timer) in self.nodes.iter() {
            timer.get_simple(name, vec);
        }
        vec.push((name.clone(), self.timer.clone()));
    }

    fn get_node(&mut self, nodes_name: &mut Vec<String>) -> *mut TimerNode {
        let node_name = nodes_name.remove(0);
        let node = self.nodes.get_mut(&node_name).unwrap();
        if nodes_name.is_empty() { return node }
        return node.get_node(nodes_name)
    }
}

struct BenchmarkInner {
    actual_node: Vec<String>,
    count: AtomicUsize,
    level: AtomicUsize,
    data: HashMap<String, TimerNode>,
}

impl BenchmarkInner {
    pub fn new() -> BenchmarkInner {
        BenchmarkInner {
            actual_node: Vec::new(),
            count: AtomicUsize::new(0),
            level: AtomicUsize::new(0),
            data: HashMap::new(),
        }
    }

    fn get_node(&mut self, nodes_name: &Vec<String>) -> *mut TimerNode {
        let mut nodes_name = nodes_name.clone();
        let node = self.data.get_mut(&nodes_name.remove(0)).unwrap();
        if nodes_name.is_empty() { return node }
        node.get_node(&mut nodes_name)
    }
}

pub struct Benchmark {
    inner: Arc<Mutex<BenchmarkInner>>,
    stored: Mutex<Vec<(String, TimerNode)>>,
    accumulator: Accumulator,
}

impl Benchmark {
    pub fn new(ratio_average: usize) -> Benchmark {
        Benchmark {
            inner: Arc::new(Mutex::new(BenchmarkInner::new())),
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
        for timer in self.inner.lock().unwrap().data.clone().into_iter() {
            data.push(timer);
        }

        *self.stored.lock().unwrap() = data;

        self.accumulator.accumulate(&self.inner.lock().unwrap().data);
    }

    pub fn get(&self) -> Vec<(String, TimerInfo)> {
        get_simple(self.stored.lock().unwrap().clone())
    }

    pub fn get_in_order(&self) -> Vec<(String, TimerInfo)> {
        let mut data = self.get();
        data.sort_unstable_by(| a, b | { a.1.count.cmp(&b.1.count) } );
        data
    }

    pub fn get_average(&self) -> Vec<(String, TimerInfo)> {
        get_simple(self.accumulator.get())
    }

    pub fn get_average_in_order(&self) -> Vec<(String, TimerInfo)> {
        let mut data = self.get_average();
        data.sort_unstable_by(| a, b | { a.1.count.cmp(&b.1.count) } );
        data

    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.actual_node.clear();
        inner.data.clear();
        inner.count.store(0, Ordering::Relaxed);
        inner.level.store(0, Ordering::Relaxed);
    }

    pub fn clear_accumulator(&self) {
        self.accumulator.clear();
    }
}

pub struct Timer {
    benchmark: Arc<Mutex<BenchmarkInner>>,
    local_node_string_list: Vec<String>,
    name: String,
    instant: std::time::Instant,
}

impl Timer {
    fn new(benchmark: Arc<Mutex<BenchmarkInner>>, name: &str) -> Timer {
        let name = name.to_owned();
        let mut benchmark_inner = benchmark.lock().unwrap();
        let count = benchmark_inner.count.fetch_add(1, Ordering::Relaxed);
        let local_node_string_list = benchmark_inner.actual_node.clone();
        let local_level = benchmark_inner.level.fetch_add(1, Ordering::Relaxed);

        let new_node = (
            name.clone(), 
            TimerNode {
                timer: TimerInfo {
                    duration: Duration::from_millis(0),
                    level: local_level,
                    count: count,
                },
                nodes: HashMap::new(),
            }
        );
        if local_node_string_list.is_empty() {
            benchmark_inner.data.insert(new_node.0, new_node.1);
        } else {
            let local_node = benchmark_inner.get_node(&local_node_string_list);
            unsafe { (*local_node).nodes.insert(new_node.0, new_node.1) };
        }
        benchmark_inner.actual_node.push(name.to_owned());

        drop(benchmark_inner);
        Timer {
            benchmark,
            local_node_string_list,
            name,
            instant: std::time::Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let mut benchmark_inner = self.benchmark.lock().unwrap();

        let mut nodes_name = self.local_node_string_list.clone();
        nodes_name.push(self.name.clone());

        let mut node = benchmark_inner.get_node(&mut nodes_name);
        unsafe { (*node).timer.duration = self.instant.elapsed() };
        benchmark_inner.level.fetch_sub(1, Ordering::Relaxed);
        benchmark_inner.actual_node.pop();
    }
}


#[derive(Clone)]
pub struct AccumulatorTimerNode {
    timer: Vec<TimerInfo>,
    nodes: HashMap<String, AccumulatorTimerNode>,
}

impl AccumulatorTimerNode {
    fn new(ratio: usize) -> AccumulatorTimerNode {
        AccumulatorTimerNode {
            timer: Vec::with_capacity(ratio),
            nodes: HashMap::new(),
        }
    }

    fn average(&self) -> TimerNode {
        let mut nodes = HashMap::new();
        if !self.nodes.is_empty() {
            for node in self.nodes.iter() {
                let name = node.0;
                let node = node.1;
                nodes.insert(name.clone(), node.average());
            }
        }

        let mut duration_final = Duration::from_millis(0);
        for time in self.timer.iter() {
            duration_final += time.duration;
        }
        duration_final /= self.timer.len() as u32;

        TimerNode {
            timer: TimerInfo {
                duration: duration_final,
                level: self.timer[0].level,
                count: self.timer[0].count,
            },
            nodes,
        }
    }

    fn accumulate(&mut self, input: &TimerNode, ratio: usize) {
        if !input.nodes.is_empty() {
            for node_input in input.nodes.iter() {
                let name_input = node_input.0;
                let timer_input = node_input.1;
                if !self.nodes.contains_key(name_input) {
                    let node = AccumulatorTimerNode::new(ratio);
                    self.nodes.insert(name_input.clone(), node);
                }
                let accumulator = self.nodes.get_mut(name_input).unwrap();
                accumulator.accumulate(timer_input, ratio);
            }
        }

        if self.timer.len() == ratio { self.timer.remove(0); }
        self.timer.push(input.timer.clone());
    }
}
pub struct Accumulator {
    data:  Mutex<HashMap<String, AccumulatorTimerNode>>,
    ratio: usize,
}

impl Accumulator {
    pub fn new(ratio: usize) -> Accumulator {
        let ratio = ratio.max(1);
        Accumulator {
            data: Mutex::new(HashMap::new()),
            ratio,
        }
    }

    pub fn get(&self) -> Vec<(String, TimerNode)> {
        let data = self.data.lock().unwrap().clone();
        let mut data_final = Vec::new();
        for node_list in data.iter() {
            let name = node_list.0;
            let node = node_list.1;
            
            data_final.push((name.clone(), node.average()));
        }
        data_final
    }
    
    pub fn accumulate(&self, input_list: &HashMap<String, TimerNode>) {
        let mut accumulator_list = self.data.lock().unwrap();
        for input in input_list.iter() {
            let name_input = input.0;
            let timer_input = input.1;
            if !accumulator_list.contains_key(name_input) {
                let ratio = self.ratio;
                let node = AccumulatorTimerNode::new(ratio);
                accumulator_list.insert(name_input.clone(), node);
            }
            let accumulator = accumulator_list.get_mut(name_input).unwrap();
            accumulator.accumulate(timer_input, self.ratio);
        }
    }

    pub fn clear(&self) {
        self.data.lock().unwrap().clear();
    }
}

fn get_simple(input: Vec<(String, TimerNode)>) -> Vec<(String, TimerInfo)> {
    let mut vec = Vec::new();
    for (name, timer) in input.iter() {
        timer.get_simple(name, &mut vec);
    }
    vec
}