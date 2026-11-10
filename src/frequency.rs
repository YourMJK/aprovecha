use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use clap::Args;


#[derive(Args)]
pub struct Arguments {
	/// Limit the output to the first n most common characters
	#[arg(long, value_name = "n")]
	max_length: Option<usize>,
	
	/// Limit the output to characters which appeared at least n times
	#[arg(long, value_name = "n")]
	min_count: Option<u64>,
	
	/// Limit the output to characters whose frequency was at least f (between 0 and 1)
	#[arg(long, value_name = "f", value_parser = validate_frequency)]
	min_freq: Option<f64>,
	
	/// The input text files from which to calculate the character frequencies
	#[arg(value_name = "FILE")]
	file_paths: Vec<PathBuf>
}

fn validate_frequency(frequency: &str) -> Result<f64, String> {
	let float = frequency.parse::<f64>()
		.map_err(|err| err.to_string())?;
	if 0.0 <= float && float <= 1.0 {
		Ok(float)
	} else {
		Err("Frequency must be a number between 0.0 and 1.0".to_string())
	}
}

pub fn run(args: Arguments) -> Result<(), String> {
	let mut occurrences: HashMap<char, u64> = HashMap::new();
	let mut total: u64 = 0;
	
	// Count all characters in input files
	for file_path in &args.file_paths {
		total += count_character_occurrences(&file_path, &mut occurrences)?;
	}
	if total == 0 {
		return Err("No characters in input".to_string());
	}
	
	// Normalize number of occurrences against total number of characters and sort in descending order
	struct Entry { char: char, count: u64, freq: f64 }
	let mut frequencies: Vec<Entry> = occurrences.into_iter()
		.map(|(char, count)| {
			Entry { char, count, freq: count as f64 / total as f64 }
		})
		.collect();
	frequencies.sort_by(|a, b| b.count.cmp(&a.count));
	
	// Filter frequencies
	let max_length = args.max_length.unwrap_or(usize::MAX);
	let min_count = args.min_count.unwrap_or(0);
	let min_freq = args.min_freq.unwrap_or(0.0);
	let frequencies_filtered = frequencies.iter()
		.take(max_length)
		.take_while(|entry| entry.count >= min_count && entry.freq >= min_freq);
	
	// Format and output frequencies as JSON dictionary
	let mut json_map = serde_json::Map::new();
	frequencies_filtered.for_each(|entry| {
		let key = entry.char.to_string();
		let value = serde_json::json!(round_freq(entry.freq));
		json_map.insert(key, value);
	});
	let json_string = serde_json::to_string_pretty(&json_map)
		.map_err(|e| format!("Could not encode output JSON: {e}"))?;
	println!("{}", json_string);
	
	Ok(())
}


fn count_character_occurrences(file: &PathBuf, occurrences: &mut HashMap<char, u64>) -> Result<u64, String> {
	// Read string from text file
	let string = fs::read_to_string(file)
		.map_err(|e| format!("Could not read input file {}: {e}", file.display()))?;
	
	// Count characters and add to map
	let mut total: u64 = 0;
	string.chars().for_each(|c| {
		*occurrences.entry(c).or_insert(0) += 1;
		total += 1;
	});
	
	Ok(total)
}


fn round_freq(freq: f64) -> f64 {
	if freq > 1.0 || freq <= 0.0 { return freq }
	const BASE: f64 = 10.0;
	
	// Round to 3 orders of magnitude
	let mut normalized = freq;
	let mut factor = 1e2;
	while normalized < 1.0 {
		normalized *= BASE;
		factor *= BASE;
	}
	
	(freq * factor).round() / factor
}
