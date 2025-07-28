use std::fs::File;
use std::io::{ BufWriter };
use serde_json;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Pair {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}


impl Pair {
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Pair {
            x0, y0, x1, y1
        }
    }

    pub fn random_new(rng_gen: &mut impl rand::Rng) -> Pair {
        let x0 = rng_gen.random_range(-3.0..=3.99);
        let y0 = rng_gen.random_range(99.0..=102.0);

        let x1 = rng_gen.random_range(-3.0..=3.99);
        let y1 = rng_gen.random_range(99.0..=102.0);

        return Pair::new(x0, y0, x1, y1);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Pairs {
    pairs: Vec<Pair>,
}

impl Pairs {
    pub fn with_capacity(capacity: usize) -> Self {
        Pairs {
            pairs: Vec::with_capacity(capacity)
        }
    }

    pub fn push(&mut self, pair: Pair) {
        self.pairs.push(pair);
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>>{
       let file = File::create(filename)?;
       let writer = BufWriter::new(file);
       serde_json::to_writer_pretty(writer, &self)?;
       Ok(())
    }
}