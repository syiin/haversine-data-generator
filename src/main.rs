mod generator;
mod haversine;
mod lexer;
mod parser;
mod profiler;
mod timer;

use clap::{Parser, Subcommand};
use generator::{Pair, Pairs};
use haversine::{read_run_metrics, reference_haversine, save_run_metrics};
use lexer::parse_file;
use parser::{JsonValue, parse_tokens};
use rand::SeedableRng;
use rand::rngs::SmallRng;

#[derive(Parser, Debug)]
#[command(
    author = "Your Name",
    version,
    about = "A tool for generating and calculating haversine distances between coordinate pairs",
    long_about = "Haversine Data Generator\n\nThis tool can generate random coordinate pairs and calculate haversine distances between them. It's useful for testing and benchmarking haversine distance calculations.\n\nThe generate command creates a JSON file with random coordinate pairs and a metrics file for validation. The calculate command reads coordinate pairs from a JSON file and computes their haversine distances, comparing against expected values."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Generate random coordinate pairs and save to JSON file
    Generate {
        /// Output file path for the generated coordinate pairs (JSON format)
        #[arg(help = "Path where the generated coordinate pairs will be saved")]
        output_file: String,
        /// Output file path for the generated metrics (TXT format)
        #[arg(help = "Path where the generated coordinate pair metrics will be saved")]
        metrics_output: String,
        // Binary output filepath for distance pairs
        #[arg(help = "Path where the generated distance pairs will be saved")]
        distance_output: String,
    },
    /// Calculate haversine distances from coordinate pairs in a JSON file
    Calculate {
        /// Input file containing coordinate pairs in JSON format
        #[arg(help = "Path to JSON file containing coordinate pairs to process")]
        input_file: String,
        /// Metrics file containing expected values for validation
        #[arg(help = "Path to metrics file with expected distance values")]
        metrics_file: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    profile_block!("Total Time");

    let cli = Cli::parse();

    match &cli.command {
        Some(Command::Generate {
            output_file,
            metrics_output,
            distance_output,
        }) => {
            generate_pairs(output_file, metrics_output, distance_output)?;
        }
        Some(Command::Calculate {
            input_file,
            metrics_file,
        }) => {
            let file = std::fs::File::open(input_file)?;
            let (seed, points, est_distance) = read_run_metrics(metrics_file)?;

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
                println!(
                    "Distance Difference: {}",
                    (actual_distance - est_distance).abs()
                );
            } else {
                println!("Error parsing JSON");
            }

            profiler::KEEPER.report();
        }
        None => {
            println!("Haversine Data Generator");
            println!("========================");
            println!();
            println!("No command specified. Use --help for detailed usage information.");
            println!();
            println!("Available commands:");
            println!("  generate <output_file>");
            println!("    Generate random haversine coordinate pairs and save to JSON file");
            println!("    Creates both data file and metrics file for validation");
            println!();
            println!("  calculate <input_file> <metrics_file>");
            println!("    Calculate haversine distances from existing JSON coordinate pairs");
            println!("    Compares calculated distances against expected values from metrics");
            println!();
            println!("Examples:");
            println!("  {} generate data.json", env!("CARGO_PKG_NAME"));
            println!(
                "  {} calculate data.json metrics.json",
                env!("CARGO_PKG_NAME")
            );
            println!();
            println!(
                "For more information, run: {} --help",
                env!("CARGO_PKG_NAME")
            );
        }
    }

    Ok(())
}

fn calculate_pairs(json: JsonValue) -> Vec<f64> {
    profile_block!("Calculate pairs");
    let JsonValue::Object(map) = json else {
        return Vec::new();
    };
    let JsonValue::Array(pairs_array) = map.get("pairs").expect("Error getting pairs array") else {
        return Vec::new();
    };

    let mut distances: Vec<f64> = Vec::with_capacity(pairs_array.len());
    for pair in pairs_array {
        let JsonValue::Object(pair_map) = pair else {
            continue;
        };

        let distance = reference_haversine(
            &Pair::new(
                get_number_from_json(pair_map.get("x0").unwrap()),
                get_number_from_json(pair_map.get("y0").unwrap()),
                get_number_from_json(pair_map.get("x1").unwrap()),
                get_number_from_json(pair_map.get("y1").unwrap()),
            ),
            6372.8,
        );
        distances.push(distance);
    }
    return distances;
}

fn get_number_from_json(json: &JsonValue) -> f64 {
    let JsonValue::Number(n) = json else {
        return 0.0;
    };
    return *n;
}

fn generate_pairs(
    file_path: &str,
    metrics_output: &str,
    distance_output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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

    let _ = pairs.save_to_file(file_path);
    let _ = save_run_metrics(
        &distances,
        seed,
        num_pairs,
        cumu_distance,
        metrics_output,
        distance_output,
    );

    Ok(())
}


