use clap::Args;
use crate::download;


// - CLI

#[derive(Args)]
pub struct Arguments {
	/// The current Next.js build ID of the website (changes regularly). Will be scraped from main site if not specified
	#[arg(short, long)]
	build_id: Option<String>,
	
	/// The username (without leading @) from which to download the recipes
	#[arg(short, long)]
	username: String
}

pub fn cli(args: Arguments) -> Result<(), String> {
	// Get build ID and configure URL template
	let build_id = download::unwrap_or_get_build_id(args.build_id)?;
	
	// Configuration
	let url_config = download::URLConfig { build_id, username: args.username };
	
	// Get list of all recipes
	let slugs = download::get_all_recipe_slugs(&url_config)
		.map_err(|e| format!("Failed to get recipe list: {e}"))?;
	
	// Print recipe list
	for slug in slugs {
		println!("{slug}");
	}
	
	Ok(())
}
