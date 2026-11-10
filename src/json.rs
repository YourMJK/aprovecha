use std::fs;
use std::path::PathBuf;
use serde_json::Value;

pub fn from_file(file: &PathBuf) -> Result<Value, String> {
	let string = fs::read_to_string(file)
		.map_err(|e| format!("Could not read file {}: {e}", file.display()))?;
	let json: Value = serde_json::from_str(&string)
		.map_err(|e| format!("Invalid JSON input: {e}"))?;
	
	Ok(json)
}
