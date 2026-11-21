use std::collections::HashMap;
use std::ops::{RangeInclusive};
use std::path::PathBuf;
use clap::Args;
use serde_json::Value;
use crate::{decode, frequency};
use crate::key::LengthHint::*;

const MIN_LENGTH: u8 = 2;
const MAX_LENGTH_DEFAULT: u8 = 32;
const PRINTABLE_ASCII: RangeInclusive<char> = ' '..='~';
const KEY_CHAR_RANGE: RangeInclusive<char> = PRINTABLE_ASCII;

enum LengthHint {
	Known(u8),
	Max(u8),
}


// - CLI

#[derive(Args)]
pub struct Arguments {
	#[command(flatten)]
	options: Options,
	
	/// Do not check if the found key actual produces valid JSON when applied to the input file
	#[arg(long)]
	no_check: bool,
	
	/// Print debug information such as confidence per found key character
	#[arg(long)]
	debug: bool,
	
	/// The input JSON file from which to extract the key
	#[arg(value_name = "RECIPE FILE")]
	input_file: PathBuf
}

#[derive(Args)]
pub struct Options {
	/// A JSON file containing absolute frequencies (probabilities) of the most common characters (see subcommand `frequency`).
	/// If not specified, the default table for English JSON output will be used
	#[arg(long = "freq-map", value_name = "FILE", verbatim_doc_comment)]
	freq_map_file: Option<PathBuf>,
	
	/// Specify the number of characters in the key (if known). Otherwise, period of input will be analyzed to determine key length
	#[arg(short, long, value_name = "n")]
	length: Option<u8>,
	
	/// Check key lengths from 1 up to n. Try increasing this number if no key could be found
	#[arg(long, value_name = "n", default_value_t = MAX_LENGTH_DEFAULT, value_parser = clap::value_parser!(u8).range(MIN_LENGTH as i64..=u8::MAX as i64))]
	max_length: u8,
}

pub fn cli(args: Arguments) -> Result<(), String> {
	let key = from_file(&args.input_file, &args.options, !args.no_check, args.debug)?;
	
	// Output key
	println!("{key}");
	
	Ok(())
}


pub fn from_file(input_file: &PathBuf, options: &Options, check: bool, debug: bool) -> Result<String, String> {
	// Set up frequency map (use default if not specified) and length hint
	let lin_freq_map = options.freq_map_file.clone().map_or(
		Ok(frequency::default_eng_json_map()),
		|file| frequency::read_map_file(&file)
	)?;
	let log_freq_map = frequency::to_log(&lin_freq_map)?;
	let length_hint = options.length.map_or(Max(options.max_length), Known);
	
	// Find key using given input file, frequency map and length hint
	let encrypted_string = decode::file_to_encrypted_string(input_file)?;
	let key = find_key(&encrypted_string, &log_freq_map, length_hint, debug)?;
	
	// Check if key produces valid JSON
	if check {
		check_key(&encrypted_string, &key)
			.map_err(|e| format!(
				"{e}\n\nTry a larger input file or specify a different frequency map if the language is not English.{}",
				if !debug { "\nUse --debug for more information." } else { "" }
			))?;
	}
	
	Ok(key)
}

fn check_key(encrypted_string: &str, key: &str) -> Result<(), String> {
	let decrypted_string = decode::encrypted_string_to_json_string(&encrypted_string, &key)
		.map_err(|e| format!("Found key failed to decode input: {e}"))?;
	let _: Value = serde_json::from_str(&decrypted_string)
		.map_err(|e| format!("Found key does not produce valid JSON on input: {e}"))?;
	Ok(())
}


// - Column Iterator

fn columns(chars: &[char], width: usize) -> impl Iterator<Item = impl Iterator<Item = char>> {
	(0..width).map(move |index| column(&chars, width, index))
}

fn column(chars: &[char], width: usize, index: usize) -> impl Iterator<Item = char> {
	chars.iter().copied().skip(index).step_by(width)
}


// - Find Key

fn find_key(encrypted_string: &str, freq_map: &frequency::LogFreqMap, length_hint: LengthHint, debug: bool) -> Result<String, String> {
	let chars: Vec<char> = encrypted_string.chars().collect();
	
	// Find key length
	let length = match length_hint {
		Known(length) => length,
		Max(max_length) => find_period(&chars, MIN_LENGTH..=max_length)?,
	} as usize;
	
	// Check whether input size is longer than key length, otherwise columns can be empty
	let input_length = chars.len();
	if debug {
		eprintln!("Key length: {length}, input size: {input_length}");
	}
	if input_length < length {
		return Err("Input has to be longer than key length".to_string());
	}
	if length == 0 {
		return Err("Key cannot be empty".to_string());
	}
	
	// Find key through column search
	let key = column_search(&chars, freq_map, length, debug);
	Ok(key)
}

fn column_search(chars: &[char], freq_map: &frequency::LogFreqMap, length: usize, debug: bool) -> String {
	columns(chars, length).enumerate()
		.map(|(index, column)| {
			// Calculate scores for each possible key character using cached column
			let column: Vec<char> = column.collect();
			let weighted_chars = KEY_CHAR_RANGE
				.map(|k| {
					WeightedChar {
						char: k,
						score: total_log_probability_after_xor(&column, k, freq_map)
					}
				});
			
			// Select character with the highest score. Additionally collect and print info about confidence in debug mode
			if debug {
				let (char, confidence) = pick_best_with_confidence(weighted_chars);
				eprintln!("Confidence for index {:02} ({char}):  {:.1} bits margin", index, confidence);
				char
			} else {
				pick_best(weighted_chars)
			}
		})
		.collect::<String>()
}

struct WeightedChar {
	char: char,
	score: f64,
}
fn pick_best(weighted_chars: impl Iterator<Item = WeightedChar>) -> char {
	weighted_chars
		.max_by(|a, b| a.score.total_cmp(&b.score))
		.map(|w| w.char)
		.unwrap()
}
fn pick_best_with_confidence(weighted_chars: impl Iterator<Item = WeightedChar>) -> (char, f64) {
	let weighted_chars = weighted_chars.collect::<Vec<_>>();
	
	// Find best and second-best chars and their log scores
	let mut best = &WeightedChar { char: '\0', score: f64::NEG_INFINITY };
	let mut second = best;
	for w in weighted_chars.iter() {
		if w.score > best.score {
			second = best;
			best = &w;
		} else if w.score > second.score {
			second = &w;
		}
	}
	
	// Calculate rival mass R as total weight of rivals relative to winner:
	// R = Σ_{i ≠ best} exp2(s_i) / exp2(s_best)
	// = Σ_{i ≠ best} exp2(s_i − s_best)
	// = Σ_{i ≠ best} (exp2(s_i − s_second) * exp2(s_second − s_best))
	// = exp2(s_second − s_best) * Σ_{i ≠ best} exp2(s_i − s_second)
	// = exp2(s_second − s_best) * tail
	let tail: f64 = weighted_chars.iter()
		.filter(|w| w.char != best.char)
		.map(|w| (w.score - second.score).exp2())
		.sum();
	
	// Calculate confidence as margin (in bits) between winner and combined rivals
	// (i.e. rivals are 2^(-margin) as likely as winner):
	// margin = -log2(R)
	// = -log2(exp2(s_second − s_best) * tail)
	// = -((s_second − s_best) + log2(tail))
	// = (-s_second + s_best) - log2(tail)
	// = s_best - s_second - log2(tail)
	let margin = best.score - second.score - tail.log2();
	
	(best.char, margin)
}


// - Find Period

fn find_period(chars: &[char], range: RangeInclusive<u8>) -> Result<u8, String> {
	// Check whether input size is sufficient for search. If a column height is less than 2, its IOC will be NaN
	let min_chars_length = (*range.end() as usize) * 2;
	if chars.len() < min_chars_length {
		return Err("Input has to be at least twice as long a maximum key length".to_string());
	}
	
	// Use average of each column's index of coincidence as a score for the period
	let scores: HashMap<u8, f64> = range.into_iter()
		.map(|length| {
			let columns = columns(chars, length as usize);
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


// - Character Scoring

/// Returns log2(P(all XORed chars are in output))
fn total_log_probability_after_xor(chars: &[char], b: char, freq_map: &frequency::LogFreqMap) -> f64 {
	chars.iter()
		.map(|a| {
			let char = char::from_u32(*a as u32 ^ b as u32);
			character_log_probability(char, freq_map)
		})
		.sum()
}

/// Returns log2(P(char is in output))
fn character_log_probability(char: Option<char>, freq_map: &frequency::LogFreqMap) -> f64 {
	let Some(char) = char else {
		return -26.5754247591;  // log2(1e-8)
	};
	if let Some(freq) = freq_map.get(&char) {
		*freq
	} else if char.is_control() && !char.is_whitespace() {
		-19.9315685693  // log2(1e-6)
	} else if PRINTABLE_ASCII.contains(&char) {
		 -7.6438561898  // log2(0.005)
	} else if char.is_alphanumeric() {
		 -8.9657842847  // log2(0.002)
	} else {
		 -9.9657842847  // log2(0.001)
	}
}


// - Period Scoring

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
