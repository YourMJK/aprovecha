mod decode;
mod frequency;

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
	/// Decode a JSON file containing the .pageProps.encodedRecipe field
	Decode(decode::Arguments),
	
	/// Analyze the character frequencies of known valid output files and return the result as a JSON dictionary
	Frequency(frequency::Arguments),
}


fn run(args: Arguments) -> Result<(), String> {
	match args.command {
		Command::Decode(args) => decode::run(args),
		Command::Frequency(args) => frequency::run(args),
	}
}

fn main() {
	let args = Arguments::parse();
	
	if let Err(e) = run(args) {
		eprintln!("Error:  {e}");
		process::exit(1);
	}
}
