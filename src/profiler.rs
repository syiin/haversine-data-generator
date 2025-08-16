use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::timer::{estimate_cpu_timer_freq, read_cpu_timer};

pub struct RecordKeeper {
    global_start: u64,
    block_profilers: Mutex<Vec<BlockProfiler>>,
    global_idx: AtomicU64,
}

pub static KEEPER : Lazy<RecordKeeper> = Lazy::new(RecordKeeper::new);

impl RecordKeeper {
    pub fn new() -> Self {
        RecordKeeper {
            global_start: unsafe { read_cpu_timer() },
            block_profilers: Mutex::new(Vec::new()),
            global_idx: AtomicU64::new(0),
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
            let inclusive_duration = record.duration;
            let exclusive_duration = record.duration.saturating_sub(record.child_duration);

            let inclusive_percentage = if total_duration > 0 {
                (inclusive_duration as f64 / total_duration as f64) * 100.0
            } else {
                0.0
            };

            let exclusive_percentage = if total_duration > 0 {
                (exclusive_duration as f64 / total_duration as f64) * 100.0
            } else {
                0.0
            };

            let inclusive_ms = inclusive_duration as f64 * 1000.0 / cpu_freq as f64;
            let exclusive_ms = exclusive_duration as f64 * 1000.0 / cpu_freq as f64;

            println!(
                "- {:<30} | Inclusive: {:>8.2} ms ({:>5.1}%) | Exclusive: {:>8.2} ms ({:>5.1}%)",
                record.name, inclusive_ms, inclusive_percentage, exclusive_ms, exclusive_percentage
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
    parent_idx: usize,
    idx: usize,
    child_duration: u64,
    duration: u64,
}

impl BlockProfiler {
    pub fn new(name: &str) -> Self {
        KEEPER.global_idx.fetch_add(1, Ordering::SeqCst);

        let current_idx = KEEPER.block_profilers.lock().unwrap().len();
        BlockProfiler {
            name: name.to_string(),
            start: unsafe { read_cpu_timer() },
            parent_idx: if current_idx > 0 { current_idx - 1 } else { 0 },
            idx: current_idx,
            child_duration: 0,
            duration: 0,
        }
    }
}

impl Drop for BlockProfiler {
    fn drop(&mut self) {
        let duration = unsafe { read_cpu_timer() } - self.start;

        // Update this profiler's duration
        KEEPER.with_block_profiler(self.idx, |block_profiler| {
            block_profiler.duration = duration;
        });

        // Update parent's child duration (only if this isn't the root profiler)
        if self.idx > 0 {
            KEEPER.with_block_profiler(self.parent_idx, |block_profiler| {
                block_profiler.child_duration += duration;
            });
        }

        KEEPER.global_idx.fetch_sub(1, Ordering::SeqCst);
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
