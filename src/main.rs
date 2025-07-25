mod generator;
mod parser;

use rand::rngs::SmallRng;
use rand::SeedableRng;
use generator::{ Pairs, Pair };
use generator::{ reference_haversine, save_run_metrics };
use parser::{ parse_file, JsonItem };

fn main() -> Result<(), Box<dyn std::error::Error>> {
    generate_pairs();
    let file = std::fs::File::open("haversine.json").unwrap();
    let tokens = parse_file(file);
    println!("Token length: {}", tokens.len());
    for token in tokens {
        println!("{}", token.format());
    }
    // parse_json("haversine.json".to_string());
    Ok(())
}

fn generate_pairs() -> Result<(), Box<dyn std::error::Error>> {
    let num_pairs = 1;
    let mut cumu_distance: f64 = 0.0;

    let seed: u64 = rand::random();
    let mut rng_gen = SmallRng::seed_from_u64(seed);

    let mut pairs = Pairs::with_capacity(num_pairs.try_into().unwrap());
    let mut distances: Vec<f64> = Vec::with_capacity(num_pairs.try_into().unwrap());
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
