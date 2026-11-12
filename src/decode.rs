use std::fs;
use std::path::PathBuf;
use clap::Args;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::Value;
use crate::json;


// - CLI

#[derive(Args)]
pub struct Arguments {
	/// The output directory path
	#[arg(short, long = "out", value_name = "DIRECTORY")]
	output: PathBuf,
	
	/// The repeated key for the XOR cipher (usually 24 characters long)
	#[arg(short, long)]
	key: String,
	
	/// Overwrite existing output files
	#[arg(short, long)]
	force: bool,
	
	/// The input JSON files to decode
	#[arg(value_name = "RECIPE FILE")]
	files: Vec<PathBuf>
}

pub fn cli(args: Arguments) -> Result<(), String> {
	// Create output directory if not exists
	let output_dir = args.output;
	let output_dir_exists = fs::metadata(&output_dir)
		.is_ok_and(|m| m.is_dir());
	if !output_dir_exists {
		fs::create_dir_all(&output_dir)
			.map_err(|e| format!("Could not create output directory: {e}"))?;
	}
	
	// Handle each input file
	for input_file in args.files {
		println!("> {}", input_file.display());
		file_to_file(&input_file, &output_dir, &args.key, args.force)?
	}
	
	Ok(())
}


// - Decoding & Decryption

fn file_to_file(input_file: &PathBuf, output_dir: &PathBuf, key: &str, overwrite: bool) -> Result<(), String> {
	// Get "encodedRecipe" value and decode it using the key
	let encrypted_string = file_to_encrypted_string(input_file)?;
	let output_json_string = encrypted_string_to_json_string(&encrypted_string, key)?;
	
	// Verify decoded string is valid JSON and get slug name
	let output_json: Value = json::from_string(&output_json_string)
		.map_err(|_| "Output is not valid JSON (wrong key?)")?;
	let name = json::get_str_value(&output_json, json::SLUG_POINTER)?;
	
	// Check if output file exists
	let output_file = output_dir.join(name).with_added_extension("json");
	if !overwrite && output_file.exists() {
		return Err(format!("Output file {} already exists", output_file.display()));
	}
	
	// Write output file
	fs::write(&output_file, output_json_string)
		.map_err(|e| format!("Could not write output file {}: {e}", output_file.display()))
}


pub fn file_to_encrypted_string(input_file: &PathBuf) -> Result<String, String> {
	// Read input JSON
	let input_json: Value = json::from_file(input_file)?;
	
	// Get "encodedRecipe" value (base64)
	let base64_string = json::get_str_value(&input_json, json::ENCODED_RECIPE_POINTER)?;
	
	// Decode base64 string
	let encrypted_bytes = STANDARD
		.decode(base64_string)
		.map_err(|e| format!("Invalid base64: {e}"))?;
	String::from_utf8(encrypted_bytes)
		.map_err(|e| format!("Invalid UTF-8: {e}"))
}


pub fn encrypted_string_to_json_string(encrypted_string: &str, key: &str) -> Result<String, String> {
	// Check key
	let key: Vec<char> = key.chars().collect();
	if key.is_empty() {
		return Err("Key must not be empty".to_string());
	}
	
	// Apply XOR per character (code point), not per byte
	encrypted_string
		.chars()
		.zip(key.iter().cycle())
		.map(|(c, k)| {
			char::from_u32(c as u32 ^ *k as u32)
				.ok_or_else(|| "XOR produced an invalid character. Wrong key?".to_string())
		})
		.collect::<Result<String, String>>()
}
