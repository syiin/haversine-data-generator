use rand::Rng;
use std::fs::File;
use std::io::BufWriter;
use serde_json;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct Pair {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}


impl Pair {
    fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Pair {
            x0, y0, x1, y1
        }
    }

    fn random_new(rng_gen: &mut impl rand::Rng) -> Pair {
        let x0 = rng_gen.random_range(-3.0..=3.99);
        let y0 = rng_gen.random_range(99.0..=102.0);

        let x1 = rng_gen.random_range(-3.0..=3.99);
        let y1 = rng_gen.random_range(99.0..=102.0);

        return Pair::new(x0, y0, x1, y1);
    }
}

#[derive(Debug, Clone, Serialize)]
struct Pairs {
    pairs: Vec<Pair>,
}

impl Pairs {
    fn new() -> Self {
        Pairs { pairs: vec![] }
    }

    fn push(&mut self, pair: Pair) {
        self.pairs.push(pair);
    }

    fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>>{
       let file = File::create(filename)?;
       let writer = BufWriter::new(file);
       serde_json::to_writer_pretty(writer, &self)?;
       Ok(())
    }
}


fn main() {
    let mut rng_gen = rand::rng();

    let mut pairs = Pairs::new();
    for _ in 0..10 {
        pairs.push(Pair::random_new(&mut rng_gen).clone());
    }
    pairs.save_to_file("haversine.json");
    println!("{:#?}", pairs);
}
