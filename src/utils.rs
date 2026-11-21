use std::path::PathBuf;
use clap::Args;


pub fn print_item(item: &str) {
	eprint!("> {item} ");
}
pub fn print_item_success() {
	eprintln!("✓");
}
pub fn print_item_error(msg: &str) {
	eprintln!("✗  ---  {msg}");
}


#[derive(Args)]
pub struct OutputConfig {
	/// Overwrite existing output files
	#[arg(short = 'f', long = "force")]
	pub overwrite: bool,
	
	/// The output directory path
	#[arg(short = 'o', long = "out", value_name = "DIRECTORY")]
	pub dir: PathBuf
}

impl OutputConfig {
	pub fn append_path(&self, path: &str) -> Self {
		Self {
			overwrite: self.overwrite,
			dir: self.dir.join(path)
		}
	}
}
