mod decode;

use std::{fs,process};
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// The output directory path
	#[arg(short, long, value_name = "DIRECTORY")]
	output: PathBuf,
	
	/// The repeated key for the XOR cipher (usually 20 characters long)
	#[arg(short, long)]
	key: String,
	
	/// Overwrite existing output files
	#[arg(short, long)]
	force: bool,
	
	/// The input JSON files to decode
	#[arg(value_name = "FILE")]
	file_paths: Vec<PathBuf>
}

fn run(args: Args) -> Result<(), String> {
	// Create output directory if not exists
	let output_dir = args.output;
	let output_dir_exists = fs::metadata(&output_dir)
		.is_ok_and(|m| m.is_dir());
	if !output_dir_exists {
		fs::create_dir_all(&output_dir)
			.map_err(|e| format!("Could not create output directory: {}", e))?;
	}
	
	for input_file in args.file_paths {
		println!("> {}", input_file.display());
		decode::from_file_to_file(&input_file, &output_dir, &args.key, args.force)?
	}
	
	Ok(())
}

fn main() {
	let args = Args::parse();
	
	if let Err(e) = run(args) {
		eprintln!("Error:  {e}");
		process::exit(1);
	}
}
