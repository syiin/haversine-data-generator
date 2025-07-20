use rand::{ SeedableRng };
use rand::rngs::StdRng;
use std::fs::File;
use std::io::{ Write, BufWriter };
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

fn radians_from_degrees(degrees: f64) -> f64 {
    0.01745329251994329577 * degrees
}

fn reference_haversine(pair: &Pair, earth_radius: f64) -> f64 {
    let mut lat0 = pair.x0;
    let lng0 = pair.y0;
    let mut lat1 = pair.x1;
    let lng1 = pair.y1;

    let d_lat = radians_from_degrees(lat1 - lat0);
    let d_lon = radians_from_degrees(lng1 - lng0);
    lat0 = radians_from_degrees(lat0);
    lat1 = radians_from_degrees(lat1);

    let a = (d_lat / 2.0).sin().powi(2) + lat0.cos() * lat1.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    earth_radius * c
}

fn save_run_metrics(distances: &Vec<f64>, seed: u64, num_pairs: u64, cumu_distance: f64) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create("haversine_metrics.txt")?;
    writeln!(file, "Seed: {}", seed)?;
    writeln!(file, "Points: {}", num_pairs)?;
    writeln!(file, "Est Distance: {}", cumu_distance)?;

    let file = File::create("haversine.f64")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, distances)?;

    println!("Seed: {}", seed);
    println!("Points: {}", num_pairs);
    println!("Est Distance: {}", cumu_distance);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let num_pairs = 10000000;
    let mut cumu_distance: f64 = 0.0;
    let seed = 42;

    let mut rng_gen = StdRng::seed_from_u64(seed);

    let mut pairs = Pairs::new();
    let mut distances: Vec<f64> = vec![];
    for _ in 0..num_pairs {
        let new_pair = Pair::random_new(&mut rng_gen);
        pairs.push(new_pair.clone());

        let distance = reference_haversine(&new_pair, 6372.8);
        distances.push(distance);
        cumu_distance += distance;
    }

    let _ = pairs.save_to_file("haversine.json");
    let _ = save_run_metrics(&distances, seed, num_pairs, cumu_distance);

    Ok(())
}
