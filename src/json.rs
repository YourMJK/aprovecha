use std::fs;
use std::path::PathBuf;
use serde_json::Value;

pub const NEXT_DATA_BUILD_ID_POINTER: &str = "/buildId";
pub const RECIPES_POINTER: &str = "/pageProps/recipes";
pub const ENCODED_RECIPE_POINTER: &str = "/pageProps/encodedRecipe";
pub const SLUG_POINTER: &str = "/slug";

pub fn from_file(file: &PathBuf) -> Result<Value, String> {
	let string = fs::read_to_string(file)
		.map_err(|e| format!("Could not read file {}: {e}", file.display()))?;
	from_string(&string)
}

pub fn from_string(string: &str) -> Result<Value, String> {
	serde_json::from_str(string)
		.map_err(|e| format!("Invalid JSON: {e}"))
}

pub fn get_str_value<'a>(json: &'a Value, path: &str) -> Result<&'a str, String> {
	json.pointer(path)
		.and_then(Value::as_str)
		.ok_or_else(|| format!("Missing string field {path}"))
}
pub fn get_array_value<'a>(json: &'a Value, path: &str) -> Result<&'a Vec<Value>, String> {
	json.pointer(path)
		.and_then(Value::as_array)
		.ok_or_else(|| format!("Missing array field {path}"))
}
