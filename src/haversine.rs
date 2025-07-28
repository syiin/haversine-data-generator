use std::fs::File;
use std::io::{ Write, BufWriter, BufReader, Read, BufRead };
use crate::generator::{ Pair };

const DEGREES_TO_RADIANS: f64 = 0.01745329251994329577;

#[inline]
fn radians_from_degrees(degrees: f64) -> f64 {
    DEGREES_TO_RADIANS * degrees
}

#[inline]
pub fn reference_haversine(pair: &Pair, earth_radius: f64) -> f64 {
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


pub fn save_run_metrics(distances: &Vec<f64>, seed: u64, num_pairs: u64, cumu_distance: f64, metrics_output: &str, distance_output: &str) -> Result<(), Box<dyn std::error::Error>> {
    { 
        let mut file = File::create(metrics_output)?;
        writeln!(file, "Seed: {}", seed)?;
        writeln!(file, "Points: {}", num_pairs)?;
        writeln!(file, "Est Distance: {}", cumu_distance)?;
    }

    {
        let file = File::create(distance_output)?;
        let mut writer = BufWriter::new(file);
        for &distance in distances {
            writer.write_all(&distance.to_le_bytes())?;
        }
    }

    println!("Seed: {}", seed);
    println!("Points: {}", num_pairs);
    println!("Est Distance: {}", cumu_distance);

    Ok(())
}

pub fn read_distances_from_file(filename: &str) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut distances = Vec::new();
    
    let mut buffer = [0u8; 8]; // f64 is 8 bytes
    while reader.read_exact(&mut buffer).is_ok() {
        let distance = f64::from_le_bytes(buffer);
        distances.push(distance);
    }
    
    Ok(distances)
}

pub fn read_run_metrics(filename: &str) -> Result<(u64, u64, f64), Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    
    let mut seed = 0;
    let mut points = 0;
    let mut est_distance = 0.0;
    
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("Seed: ") {
            seed = line[6..].parse()?;
        } else if line.starts_with("Points: ") {
            points = line[8..].parse()?;
        } else if line.starts_with("Est Distance: ") {
            est_distance = line[14..].parse()?;
        }
    }
    
    Ok((seed, points, est_distance))
}
