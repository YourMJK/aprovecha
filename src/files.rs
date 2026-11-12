use std::fs;
use std::path::PathBuf;

/// Create output directory if not exists.
/// Returns `true` if directory was created and `false` if it already exists
pub fn create_output_dir(path: &PathBuf) -> Result<bool, String> {
	let exists = fs::metadata(&path)
		.is_ok_and(|m| m.is_dir());
	
	if !exists {
		fs::create_dir_all(&path)
			.map_err(|e| format!("Could not create output directory: {e}"))?;
		Ok(true)
	} else {
		Ok(false)
	}
}
