use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::timer::{estimate_cpu_timer_freq, read_cpu_timer};

pub struct TimeRecord {
    name: String,
    duration: u64,
}

pub struct RecordKeeper {
    global_start: u64,
    records: Mutex<Vec<TimeRecord>>,
}

pub static KEEPER : Lazy<RecordKeeper> = Lazy::new(RecordKeeper::new);

impl RecordKeeper {
    pub fn new() -> Self {
        RecordKeeper {
            global_start: unsafe { read_cpu_timer() },
            records: Mutex::new(Vec::new()),
        }
    }

    pub fn insert_record(&self, record: TimeRecord) {
        let mut records = self.records.lock().unwrap();
        records.push(record);
    }

    pub fn report(&self) {
        let records = self.records.lock().unwrap();
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

pub struct BlockProfiler {
    name: String,
    start: u64,
}

impl BlockProfiler {
    pub fn new(name: &str) -> Self {
        BlockProfiler {
            name: name.to_string(),
            start: unsafe { read_cpu_timer() },
        }
    }
}

impl Drop for BlockProfiler {
    fn drop(&mut self) {
        let report = TimeRecord {
            name: self.name.clone(),
            duration: unsafe { read_cpu_timer() } - self.start,
        };
        KEEPER.insert_record(report);
    }
}

#[macro_export]
macro_rules! profile_block {
    ($name:expr) => {
        let _profiler = crate::profiler::BlockProfiler::new($name);
    };
    () => {
        let _profiler = crate::profiler::BlockProfiler::new(format!("{}:{}", file!(), line!()));
    };
}
