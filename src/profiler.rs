use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::timer::{estimate_cpu_timer_freq, read_cpu_timer};

pub struct RecordKeeper {
    global_start: u64,
    block_profilers: Mutex<Vec<BlockProfiler>>,
}

pub static KEEPER : Lazy<RecordKeeper> = Lazy::new(RecordKeeper::new);

impl RecordKeeper {
    pub fn new() -> Self {
        RecordKeeper {
            global_start: unsafe { read_cpu_timer() },
            block_profilers: Mutex::new(Vec::new()),
        }
    }

    pub fn insert_block_profiler(&self, block_profiler: BlockProfiler) {
        let mut block_profilers = self.block_profilers.lock().unwrap();
        block_profilers.push(block_profiler);
    }

    pub fn with_block_profiler<F, R>(&self, idx: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut BlockProfiler) -> R,
    {
        let mut block_profilers = self.block_profilers.lock().unwrap();
        if idx < block_profilers.len() {
            Some(f(&mut block_profilers[idx]))
        } else {
            None
        }
    }

    pub fn report(&self) {
        let records = self.block_profilers.lock().unwrap();
        let total_duration = unsafe { read_cpu_timer() } - self.global_start;
        let cpu_freq = estimate_cpu_timer_freq();

        println!("\n--- Profiling Report ---");
        println!("Total execution time: {:.2} ms ({} cycles)\n",
                 total_duration as f64 * 1000.0 / cpu_freq as f64, total_duration);

        for record in records.iter() {
            let percentage = if total_duration > 0 {
                (record.duration as f64 / total_duration as f64) * 100.0
            } else {
                0.0
            };
            let duration_ms = record.duration as f64 * 1000.0 / cpu_freq as f64;
            println!(
                "- {:<30} | Time: {:>10.2} ms | Cycles: {:>12} | Percentage: {:>5.1}%",
                record.name, duration_ms, record.duration, percentage
            );
        }

        println!("Estimated CPU frequency: {} Hz ({:.2} MHz)", cpu_freq, cpu_freq as f64 / 1_000_000.0);
        println!("------------------------\n");
    }
}

#[derive(Clone)]
pub struct BlockProfiler {
    name: String,
    start: u64,
    idx: usize,
    duration: u64,
}

impl BlockProfiler {
    pub fn new(name: &str) -> Self {
        BlockProfiler {
            name: name.to_string(),
            start: unsafe { read_cpu_timer() },
            idx: KEEPER.block_profilers.lock().unwrap().len(),
            duration: 0,
        }
    }
}

impl Drop for BlockProfiler {
    fn drop(&mut self) {
        KEEPER.with_block_profiler(self.idx, |block_profiler| {
            block_profiler.duration = unsafe { read_cpu_timer() } - self.start;
        });
    }
}


#[macro_export]
macro_rules! profile_block {
    ($name:expr) => {
        let _profiler = crate::profiler::BlockProfiler::new($name);
        crate::profiler::KEEPER.insert_block_profiler(_profiler.clone());
    };
    () => {
        let _profiler = crate::profiler::BlockProfiler::new(format!("{}:{}", file!(), line!()));
        crate::profiler::KEEPER.insert_block_profiler(_profiler.clone());
    };
}
