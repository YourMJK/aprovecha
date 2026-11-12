mod download;
mod key;
mod decode;
mod frequency;
mod json;
mod files;

use std::process;
use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
	#[command(subcommand)]
	command: Command
}

#[derive(Subcommand)]
enum Command {
	/// Download JSON recipe files from provecho.co
	Download(download::Arguments),
	
	/// Extract the encryption key from a JSON recipe file through statistical analysis
	Key(key::Arguments),
	
	/// Decode a JSON recipe file using the encryption key
	Decode(decode::Arguments),
	
	/// Analyze the character frequencies of known valid output files and return the result as a JSON dictionary
	Frequency(frequency::Arguments),
}


fn run(args: Arguments) -> Result<(), String> {
	match args.command {
		Command::Download(args) => download::cli(args),
		Command::Key(args) => key::cli(args),
		Command::Decode(args) => decode::cli(args),
		Command::Frequency(args) => frequency::cli(args),
	}
}

fn main() {
	let args = Arguments::parse();
	
	if let Err(e) = run(args) {
		eprintln!("Error:  {e}");
		process::exit(1);
	}
}
