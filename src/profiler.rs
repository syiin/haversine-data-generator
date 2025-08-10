use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct TimeRecord {
    name: String,
    duration: Duration,
}

pub struct RecordKeeper {
    global_start: Instant,
    records: Mutex<Vec<TimeRecord>>,
}

pub static KEEPER : Lazy<RecordKeeper> = Lazy::new(RecordKeeper::new);

impl RecordKeeper {
    pub fn new() -> Self {
        RecordKeeper {
            global_start: Instant::now(),
            records: Mutex::new(Vec::new()),
        }
    }

    pub fn insert_record(&self, record: TimeRecord) {
        let mut records = self.records.lock().unwrap();
        records.push(record);
    }

    pub fn report(&self) {
        let records = self.records.lock().unwrap();
        let total_duration = self.global_start.elapsed();

        println!("\n--- Profiling Report ---");
        println!("Total execution time: {:.2?}\n", total_duration);

        for record in records.iter() {
            let percentage = if total_duration.as_nanos() > 0 {
                (record.duration.as_nanos() as f64 / total_duration.as_nanos() as f64) * 100.0
            } else {
                0.0
            };
            println!(
                "- {:<30} | Time: {:>10.2?} | Percentage: {:>5.1}%",
                record.name, record.duration, percentage
            );
        }
        println!("------------------------\n");
    }
}

pub struct BlockProfiler {
    name: String,
    start: Instant,
}

impl BlockProfiler {
    pub fn new(name: &str) -> Self {
        BlockProfiler {
            name: name.to_string(),
            start: Instant::now(),
        }
    }
}

impl Drop for BlockProfiler {
    fn drop(&mut self) {
        let report = TimeRecord {
            name: self.name.clone(),
            duration: self.start.elapsed(),
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
