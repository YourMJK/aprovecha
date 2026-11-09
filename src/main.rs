mod decode;

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
}


fn run(args: Arguments) -> Result<(), String> {
	match args.command {
		Command::Decode(args) => {
			decode::run(args)
		},
	}
}

fn main() {
	let args = Arguments::parse();
	
	if let Err(e) = run(args) {
		eprintln!("Error:  {e}");
		process::exit(1);
	}
}
