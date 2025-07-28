mod generator;
mod parser;
mod lexer;
mod haversine;

use rand::rngs::SmallRng;
use rand::SeedableRng;
use generator::{ Pairs, Pair };
use haversine::{ reference_haversine, save_run_metrics, read_distances_from_file, read_run_metrics };
use lexer::{ parse_file };
use parser::{ parse_tokens, JsonValue };

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // generate_pairs()?;
    let file = std::fs::File::open("haversine.json")?;
    let (seed, points, est_distance) = read_run_metrics("haversine_metrics.txt")?;
    let tokens = parse_file(file);

    println!("Seed: {}", seed);
    println!("Points: {}", points);
    println!("Est Distance: {}", est_distance);

    let json = parse_tokens(&tokens);
    if let Some(json_value) = json {
        let distances: Vec<f64> = calculate_pairs(json_value);
        let actual_distance: f64 = distances.iter().sum();

        println!("Actual Distance: {}", actual_distance);
        println!("Expected Distance: {}", est_distance);
        println!("Distance Difference: {}", (actual_distance - est_distance).abs());

    } else {
        println!("Error parsing JSON");
    }
    
    Ok(())
}

fn calculate_pairs(json: JsonValue) -> Vec<f64> {
    let JsonValue::Object(map) = json else { return Vec::new(); };
    let JsonValue::Array(pairs_array) = map.get("pairs").expect("Error getting pairs array") else { return Vec::new(); };

    let mut distances: Vec<f64> = Vec::new();
    for pair in pairs_array {
        let JsonValue::Object(pair_map) = pair else { continue; };
        let x0 = pair_map.get("x0").unwrap();
        let y0 = pair_map.get("y0").unwrap();
        let x1 = pair_map.get("x1").unwrap();
        let y1 = pair_map.get("y1").unwrap();

        let pair = Pair::new(
            get_number_from_json(x0), 
            get_number_from_json(y0), 
            get_number_from_json(x1), 
            get_number_from_json(y1)
        );
        let distance = reference_haversine(&pair, 6372.8);
        distances.push(distance);
    }
    return distances;
}

fn get_number_from_json(json: &JsonValue) -> f64 {
    let JsonValue::Number(n) = json else { return 0.0; };
    return *n;
}

fn generate_pairs() -> Result<(), Box<dyn std::error::Error>> {
    let num_pairs = 10000000;
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
