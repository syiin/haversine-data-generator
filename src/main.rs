mod generator;
mod parser;
mod lexer;
mod haversine;
mod timer;

use clap::{Parser, Subcommand, Arg};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use generator::{ Pairs, Pair };
use haversine::{ reference_haversine, save_run_metrics, read_run_metrics };
use lexer::{ parse_file };
use parser::{ parse_tokens, JsonValue };
use timer::{ read_cpu_timer, estimate_cpu_timer_freq };



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
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prof_begin = unsafe { read_cpu_timer() };
    let mut prof_read = 0;
    let mut prof_misc_setup = 0;
    let mut prof_sum = 0;
    let mut prof_misc_output = 0;

    let cli = Cli::parse();

    match &cli.command {
        Some(Command::Generate { output_file , metrics_output, distance_output}) => {
            generate_pairs(output_file, metrics_output, distance_output)?;
        },
        Some(Command::Calculate { input_file, metrics_file }) => {
            prof_read = unsafe { read_cpu_timer() };
            let file = std::fs::File::open(input_file)?;
            let (seed, points, est_distance) = read_run_metrics(metrics_file)?;
            
            prof_misc_setup = unsafe { read_cpu_timer() };
            let tokens = parse_file(file);

            println!("Seed: {}", seed);
            println!("Points: {}", points);
            println!("Est Distance: {}", est_distance);

            let prof_parse = unsafe { read_cpu_timer() };
            let json = parse_tokens(&tokens);
            if let Some(json_value) = json {
                let distances: Vec<f64> = calculate_pairs(json_value);
                
                prof_sum = unsafe { read_cpu_timer() };
                let actual_distance: f64 = distances.iter().sum();

                prof_misc_output = unsafe { read_cpu_timer() };
                println!("Actual Distance: {}", actual_distance);
                println!("Expected Distance: {}", est_distance);
                println!("Distance Difference: {}", (actual_distance - est_distance).abs());
            } else {
                println!("Error parsing JSON");
            }

            let prof_end = unsafe { read_cpu_timer() };
            let total_cpu_elapsed = prof_end - prof_begin;

            let cpu_freq = estimate_cpu_timer_freq();

            println!("Total time: {} \n(CPU freq: {}) \n ", 1000 * total_cpu_elapsed / cpu_freq, cpu_freq );

            print_time_elapsed("Startup", total_cpu_elapsed, prof_begin, prof_read);
            print_time_elapsed("Read", total_cpu_elapsed, prof_read, prof_misc_setup);
            print_time_elapsed("Misc Setup", total_cpu_elapsed, prof_misc_setup, prof_parse);
            print_time_elapsed("Parse", total_cpu_elapsed, prof_parse, prof_sum);
            print_time_elapsed("Sum", total_cpu_elapsed, prof_sum, prof_misc_output);
            print_time_elapsed("Misc Output", total_cpu_elapsed, prof_misc_output, prof_end);
        },
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
            println!("  {} calculate data.json metrics.json", env!("CARGO_PKG_NAME"));
            println!();
            println!("For more information, run: {} --help", env!("CARGO_PKG_NAME"));
        }
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

fn generate_pairs(file_path: &str, metrics_output: &str, distance_output: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    let _ = save_run_metrics(&distances, seed, num_pairs, cumu_distance, metrics_output, distance_output);

    Ok(())
}

fn print_time_elapsed(label: &str, total_cpu_elapsed: u64, prof_begin: u64, prof_end: u64) {
    let elapsed: f64 = prof_end as f64 - prof_begin as f64;
    let percent: f64 = 100.0 * (elapsed / total_cpu_elapsed as f64);
    println!("{}: {}, ({}%)", label, elapsed, percent);
}