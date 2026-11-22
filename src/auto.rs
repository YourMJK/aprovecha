use std::fs;
use std::path::PathBuf;
use clap::Args;
use crate::{decode, download, key, files};
use crate::download::{DownloadOptions, URLOptions};
use crate::utils::OutputConfig;


// - CLI

#[derive(Args)]
pub struct Arguments {
	#[command(flatten)]
	url_options: URLOptions,
	
	#[command(flatten)]
	download_options: DownloadOptions,
	
	#[command(flatten)]
	key_options: key::Options,
	
	#[command(flatten)]
	output_config: OutputConfig
}

pub fn cli(args: Arguments) -> Result<(), String> {
	// Get build ID
	print_heading("Getting build ID");
	let (url_config, download_config) = download::get_config(args.url_options, args.download_options)?;
	
	// Get list of all available recipes
	print_heading(&format!("Getting list of recipes for user \"{}\"", url_config.username));
	let slugs = download::get_all_recipe_slugs(&url_config)
		.map_err(|e| format!("Failed to get recipe list: {e}"))?;
	
	// Create output directory structure
	print_heading("Creating directory structure");
	let user_output = args.output_config.append_path(&url_config.username);
	let download_output = user_output.append_path("encoded");
	let decode_output = user_output.append_path("decoded");
	files::create_output_dir(&user_output.dir)?;
	files::create_output_dir(&download_output.dir)?;
	files::create_output_dir(&decode_output.dir)?;
	
	// Download all recipes in list and save output paths
	print_heading("Downloading encoded recipes");
	let encoded_files = download::save_recipes(&url_config, &download_config, &download_output, &slugs)
		.into_iter()
		.map(|(path, _)| path)
		.collect::<Vec<_>>();
	if encoded_files.is_empty() {
		return Err("No encoded recipes saved to proceed with".to_string());
	}
	
	// Select largest encoded file to use for key finding
	print_heading("Finding key");
	let largest_encoded_file = largest_file(&encoded_files);
	let key = key::from_file(largest_encoded_file, &args.key_options, true, false)?;
	
	// Decode files
	print_heading("Decoding recipes");
	decode::files(&encoded_files, &key, &decode_output);
	
	Ok(())
}

fn print_heading(string: &str) {
	println!("{string} ...");
}

fn largest_file(files: &[PathBuf]) -> &PathBuf {
	files.iter()
		.map(|path| {
			let size = fs::metadata(path).map_or(0, |m| m.len());
			(path, size)
		})
		.max_by_key(|(_, size)| *size)
		.unwrap().0
}
