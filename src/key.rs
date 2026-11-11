use std::collections::HashMap;
use std::ops::{RangeInclusive};
use std::path::PathBuf;
use clap::Args;
use serde_json::Value;
use crate::{decode, frequency};
use crate::key::LengthHint::{Known, Max};

const MIN_LENGTH: u8 = 2;
const PRINTABLE_ASCII: RangeInclusive<char> = ' '..='~';
const KEY_CHAR_RANGE: RangeInclusive<char> = PRINTABLE_ASCII;

enum LengthHint {
	Known(u8),
	Max(u8),
}


// - CLI

#[derive(Args)]
pub struct Arguments {
	/// A JSON file containing absolute frequencies of the most common characters
	#[arg(long = "freq-map", value_name = "FILE")]
	freq_map_file: PathBuf,
	
	/// Specify the number of characters in the key (if known). Otherwise, period of input will be analyzed to determine key length
	#[arg(short, long, value_name = "n")]
	length: Option<u8>,
	
	/// Check key lengths from 1 up to n. Try increasing this number if no key could be found
	#[arg(long, value_name = "n", default_value_t = 32, value_parser = clap::value_parser!(u8).range(MIN_LENGTH as i64..=u8::MAX as i64))]
	max_length: u8,
	
	/// Do not check if the found key actual produces valid JSON when applied to the input file
	#[arg(long)]
	no_check: bool,
	
	/// The input JSON file from which to extract the key
	#[arg(value_name = "RECIPE FILE")]
	input_file: PathBuf
}

pub fn cli(args: Arguments) -> Result<(), String> {
	let length_hint = args.length.map_or(Max(args.max_length), Known);
	
	// Find key using given input file, frequency file and length hint
	let freq_map = frequency::read_map_file(&args.freq_map_file)?;
	let encrypted_string = decode::file_to_encrypted_string(&args.input_file)?;
	let key = find_key(&encrypted_string, &freq_map, length_hint)?;
	
	// Check if key produces valid JSON
	if !args.no_check {
		let decrypted_string = decode::encrypted_string_to_json_string(&encrypted_string, &key)
			.map_err(|e| format!("Found key failed to decode input: {e}"))?;
		let _: Value = serde_json::from_str(&decrypted_string)
			.map_err(|e| format!("Found key does not produce valid JSON on input: {e}"))?;
	}
	
	// Output key
	println!("{}", key);
	
	Ok(())
}


// - Find Key

fn find_key(encrypted_string: &str, freq_map: &frequency::FreqMap, length_hint: LengthHint) -> Result<String, String> {
	let chars: Vec<char> = encrypted_string.chars().collect();
	
	// Find key length
	let length = match length_hint {
		Known(length) => length,
		Max(max_length) => find_period(&chars, MIN_LENGTH..=max_length)?,
	};
	
	let key = columns(&chars, length as usize)
		.map(|column| {
			let column: Vec<char> = column.collect();
			KEY_CHAR_RANGE
				.map(|k| (k, total_character_score_after_xor(&column, k, &freq_map)))
				.max_by(|a, b| a.1.total_cmp(&b.1))
				.map(|(k, _)| k)
				.unwrap()
		})
		.collect::<String>();
	
	Ok(key)
}

fn find_period(chars: &[char], range: RangeInclusive<u8>) -> Result<u8, String> {
	// Check whether input size is sufficient for search. If a column height is less than 2, its IOC will be NaN
	let min_chars_length = (*range.end() as usize) * 2;
	if chars.len() < min_chars_length {
		return Err("Input has to be at least twice as long a maximum key length".to_string());
	}
	
	// Use average of each column's index of coincidence as a score for the period
	let scores: HashMap<u8, f64> = range.into_iter()
		.map(|length| {
			let columns = columns(&chars, length as usize);
			let score = average_index_of_coincidence(columns);
			(length, score)
		})
		.collect();
	
	// Select the shortest period of the top 5% matches
	let best_score = scores.values().max_by(|a, b| a.total_cmp(b)).unwrap();
	let threshold = best_score * 0.95;
	let shortest_period = scores.iter()
		.filter(|(_, score)| **score > threshold)
		.map(|(length, _)| length)
		.min()
		.unwrap();
	
	Ok(*shortest_period)
}


// - Column Iterator

fn columns(chars: &[char], width: usize) -> impl Iterator<Item = impl Iterator<Item = char>> {
	(0..width).map(move |index| column(&chars, width, index))
}
fn column(chars: &[char], width: usize, index: usize) -> impl Iterator<Item = char> {
	chars[index..].iter().copied().step_by(width)
}


// - Scoring

fn total_character_score_after_xor(chars: &[char], b: char, freq_map: &frequency::FreqMap) -> f64 {
	let mut score = 0.0;
	for a in chars {
		let char = char::from_u32(*a as u32 ^ b as u32);
		score += character_score(char, &freq_map);
	}
	score
}
fn character_score(char: Option<char>, freq_map: &frequency::FreqMap) -> f64 {
	let Some(char) = char else {
		return -1.0
	};
	if let Some(freq) = freq_map.get(&char) {
		*freq
	} else if char.is_control() && !char.is_whitespace() {
		-1.0
	} else if PRINTABLE_ASCII.contains(&char) || char.is_alphanumeric() {
		0.005
	} else {
		0.0
	}
}

fn average_index_of_coincidence(columns: impl Iterator<Item = impl Iterator<Item = char>>) -> f64 {
	let iocs = columns
		.map(index_of_coincidence);
	let (sum, count) = iocs.fold((0.0, 0usize), |(s, n), x| (s + x, n + 1));
	sum / count as f64
}

fn index_of_coincidence<I>(chars: I) -> f64
where I: IntoIterator<Item = char> {
	// Count occurrences and length
	let mut occurrences: HashMap<char, usize> = HashMap::new();
	let mut n = 0;
	chars.into_iter().for_each(|c| {
		*occurrences.entry(c).or_insert(0) += 1;
		n += 1;
	});
	if n < 2 { return f64::NAN }
	
	// Calculate pairs
	let matching_pairs: usize = occurrences.into_values()
		.map(|v| v * (v-1))
		.sum();
	let total_pairs = n * (n-1);
	
	matching_pairs as f64 / total_pairs as f64
}
