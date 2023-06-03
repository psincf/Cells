use crate::TimerInfo;
use std::collections::HashMap;
use std::time::Duration;
use std::sync::Mutex;

pub struct Accumulator {
    data: Mutex<HashMap<String, Vec<TimerInfo>>>,
    data_used: Mutex<HashMap<String, Vec<TimerInfo>>>,
    ratio: usize,
}

impl Accumulator {
    pub fn new(ratio: usize) -> Accumulator {
        let ratio = ratio.max(1);
        Accumulator {
            data: Mutex::new(HashMap::new()),
            data_used: Mutex::new(HashMap::new()),
            ratio,
        }
    }

    pub fn get(&self) -> Vec<(String, TimerInfo)> {
        let data = self.data_used.lock().unwrap().clone();
        let mut data_final = Vec::new();
        for bench in data.iter() {
            let name = bench.0;
            let times = bench.1;
            let mut result = Duration::from_nanos(0);
            for time in times {
                result += time.duration;
            }
            result /= times.len() as u32;
            data_final.push(
                (
                    name.clone(),
                    TimerInfo {
                        duration: result,
                        level: times[0].level,
                        count: times[0].count,
                    }
                )
            );
        }
        data_final
    }
    
    pub fn accumulate(&self, input: &Mutex<HashMap<String, TimerInfo>>) {
        let mut average = self.data.lock().unwrap();
        let input = input.lock().unwrap();
        for input_new in input.iter() {
            let name_new = input_new.0;
            let timer_new = input_new.1;
            if !average.contains_key(name_new) {
                let ratio = self.ratio;
                let vec = Vec::with_capacity(ratio);
                average.insert(name_new.clone(), vec);
            }
            let data_bench = average.get_mut(name_new).unwrap();
            if data_bench.len() == self.ratio { data_bench.remove(0); }
            data_bench.push(timer_new.clone());
        }

        *self.data_used.lock().unwrap() = average.clone();
        let mut data_used = self.data_used.lock().unwrap();
        for timer in data_used.clone().iter() {
            if !input.iter().any(|i| { i.0 == timer.0 }) {
                data_used.remove(timer.0).unwrap();
            }
        }
    }

    pub fn clear(&self) {
        self.data.lock().unwrap().clear();
    }
}