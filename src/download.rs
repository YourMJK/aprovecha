use std::cell::{Cell, RefCell};
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use clap::Args;
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts};
use html5ever::tokenizer::states::RawKind;
use crate::{files, json};

const BASE_URL: &str = "https://www.provecho.co";
const MAIN_PAGE_URL: &str = constcat::concat!(BASE_URL, "/platform");
const NEXT_DATA_ID: &str = "__NEXT_DATA__";

struct URLConfig { build_id: String, username: String }
struct OutputConfig { dir: PathBuf, overwrite: bool }


// - CLI

#[derive(Args)]
pub struct Arguments {
	/// The current Next.js build ID of the website (changes regularly). Will be scraped from main site if not specified
	#[arg(short, long)]
	build_id: Option<String>,
	
	/// The username (without leading @) from which to download the recipes
	#[arg(short, long)]
	username: String,
	
	/// How long to wait between each request (as to not upset the server) in milliseconds
	#[arg(short, long, value_name = "ms", default_value_t = 1000)]
	interval: u16,
	
	/// Overwrite existing output files
	#[arg(short, long)]
	force: bool,
	
	/// The output directory path
	#[arg(short, long = "out", value_name = "DIRECTORY")]
	output_dir: PathBuf
}

pub fn cli(args: Arguments) -> Result<(), String> {
	// Get build ID and configure URL template
	let build_id = unwrap_or_get_build_id(args.build_id)?;
	
	// Configuration
	let url_config = URLConfig { build_id, username: args.username };
	let output_config = OutputConfig { dir: args.output_dir, overwrite: args.force };
	let interval = Duration::from_millis(args.interval as u64);
	
	// Get list of all recipes
	let slugs = get_all_recipe_slugs(&url_config)
		.map_err(|e| format!("Failed to get recipe list: {e}"))?;
	
	// Create output directory if not exists
	files::create_output_dir(&output_config.dir)?;
	
	// Download all recipes in list
	save_recipes(&url_config, &output_config, &slugs, interval)
}


fn unwrap_or_get_build_id(build_id: Option<String>) -> Result<String, String> {
	if let Some(build_id) = build_id {
		return Ok(build_id);
	}
	println!("No build ID specified, scraping current value from main site ...");
	
	let build_id = get_build_id()
		.map_err(|e| format!("Failed to get build ID: {e}"))?;
	println!("Build ID: {build_id}");
	
	Ok(build_id)
}


// - Download

fn save_recipes(url_config: &URLConfig, output_config: &OutputConfig, slugs: &[String], interval: Duration) -> Result<(), String> {
	for slug in slugs {
		println!("> {}", slug);
		save_recipe(url_config, output_config, slug)?;
		sleep(interval);
	}
	Ok(())
}

fn save_recipe(url_config: &URLConfig, output_config: &OutputConfig, slug: &str) -> Result<(), String> {
	let filename = format!("{slug}.json");
	
	// Check if output file exists
	let output_file = output_config.dir.join(&filename);
	if !output_config.overwrite && output_file.exists() {
		return Err(format!("Output file {} already exists", output_file.display()));
	}
	
	// Get file contents
	let contents = get_user_data(url_config, &filename)?;
	
	// Write output file
	fs::write(&output_file, contents)
		.map_err(|e| format!("Could not write output file {}: {e}", output_file.display()))
}


// - Provecho Requests

fn get_all_recipe_slugs(config: &URLConfig) -> Result<Vec<String>, String> {
	// Get recipes.json
	let json_string = get_user_data(config, "recipes.json")?;
	let json = json::from_string(&json_string)?;
	
	// Collect all .pageProps.recipes[].slug
	json::get_array_value(&json, json::RECIPES_POINTER)?
		.into_iter()
		.map(|item| {
			json::get_str_value(item, json::SLUG_POINTER).map(String::from)
		})
		.collect::<Result<Vec<_>, _>>()
}

fn get_build_id() -> Result<String, String> {
	// Get HTML from main page
	let html = send_request(MAIN_PAGE_URL)?;
	
	// Setup HTML tokenizer with NextDataFinder to get contents of <script id="__NEXT_DATA__"> tag
	let tokenizer = Tokenizer::new(
		NextDataFinder::default(),
		TokenizerOpts::default()
	);
	
	// Parse HTML
	let input = BufferQueue::default();
	input.push_back(StrTendril::from_slice(&html));
	let _ = tokenizer.feed(&input);
	tokenizer.end();
	
	// Parse JSON contents of tag to get the .buildId
	let json_string = tokenizer.sink.content.take()
		.ok_or(format!("No \"{NEXT_DATA_ID}\" tag found in response"))?;
	let json = json::from_string(&json_string)
		.map_err(|_| format!("Invalid JSON in \"{NEXT_DATA_ID}\" tag"))?;
	let build_id = json::get_str_value(&json, json::NEXT_DATA_BUILD_ID_POINTER)?;
	
	Ok(build_id.to_string())
}

fn get_user_data(config: &URLConfig, filename: &str) -> Result<String, String> {
	let url = format!("{BASE_URL}/_next/data/{}/%40{}/{filename}", config.build_id, config.username);
	send_request(&url)
}


// - HTML Parsing

#[derive(Default)]
struct NextDataFinder {
	found_tag: Cell<bool>,
	content: RefCell<Option<String>>
}

impl TokenSink for NextDataFinder {
	type Handle = ();
	
	fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
		match token {
			Token::TagToken(tag) if tag.name.eq("script") => {
				match tag.kind {
					TagKind::StartTag => {
						let has_next_js_id = tag.attrs.iter()
							.any(|a|{
								a.name.local.eq("id") && a.value.eq(NEXT_DATA_ID)
							});
						if has_next_js_id {
							self.found_tag.set(true);
							self.content.borrow_mut().get_or_insert_with(String::new);
							return TokenSinkResult::RawData(RawKind::ScriptData);
						}
					}
					TagKind::EndTag => {
						self.found_tag.set(false);
					}
				}
			}
			Token::CharacterTokens(string) if self.found_tag.get() => {
				if let Some(buf) = self.content.borrow_mut().as_mut() {
					buf.push_str(&string);
				}
			}
			_ => {}
		}
		TokenSinkResult::Continue
	}
}


// - HTTP Requests

fn send_request(url: &str) -> Result<String, String> {
	let response = minreq::get(url).send()
		.map_err(|e| format!("Failed to send request for {url}: {e}"))?;
	if !status_is_ok(response.status_code) {
		return Err(format!("Responded with status {} for {url}", response.status_code));
	}
	let body = response.as_str()
		.map_err(|e| format!("Response from {url} is not valid UTF-8: {e}"))?;
	
	Ok(body.to_string())
}

fn status_is_ok(code: u16) -> bool {
	code >= 200 && code < 300
}
