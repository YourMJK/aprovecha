use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::{fs, io};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};
use clap::Args;
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts};
use html5ever::tokenizer::states::RawKind;
use crate::{files, json, utils};
use crate::utils::OutputConfig;

const BASE_URL: &str = "https://www.provecho.co";
const MAIN_PAGE_URL: &str = constcat::concat!(BASE_URL, "/platform");
const NEXT_DATA_ID: &str = "__NEXT_DATA__";

pub struct URLConfig {
	pub build_id: String,
	pub username: String
}
pub struct DownloadConfig {
	pub interval: Duration
}


// - CLI

#[derive(Args)]
pub struct Arguments {
	#[command(flatten)]
	url_options: URLOptions,
	
	#[command(flatten)]
	download_options: DownloadOptions,
	
	#[command(flatten)]
	output_config: OutputConfig,
	
	/// The (slug) names of the recipes to download.
	/// If none are provided, all available recipes will be downloaded
	#[arg(value_name = "NAME", verbatim_doc_comment)]
	slugs: Vec<String>
}

#[derive(Args)]
pub struct URLOptions {
	/// The current Next.js build ID of the website (changes regularly).
	/// Will be scraped from main site if not specified
	#[arg(short, long, verbatim_doc_comment)]
	build_id: Option<String>,
	
	/// The username (without leading @) from which to download the recipes
	#[arg(short, long)]
	username: String,
}

#[derive(Args)]
pub struct DownloadOptions {
	/// How long to wait between each request (as to not upset the server) in milliseconds
	#[arg(short, long, value_name = "ms", default_value_t = 1000)]
	interval: u16,
}

pub fn cli(args: Arguments) -> Result<(), String> {
	// Get build ID & Prepare configuration
	let (url_config, download_config) = get_config(args.url_options, args.download_options)?;
	let output_config = args.output_config;
	
	// Get list of all available recipes
	let mut slugs = get_all_recipe_slugs(&url_config)
		.map_err(|e| format!("Failed to get recipe list: {e}"))?;
	
	// Check list of requested recipes against available recipes
	if !args.slugs.is_empty() {
		let available_slugs = slugs.iter().cloned().collect::<HashSet<_>>();
		slugs.clear();
		let mut invalid_slug = false;
		
		for slug in args.slugs {
			if available_slugs.contains(&slug) {
				slugs.push(slug);
			} else {
				eprintln!("No recipe named \"{slug}\" found");
				invalid_slug = true;
			}
		}
		
		if invalid_slug {
			eprintln!("Use the \"list\" subcommand for a list of available recipe names");
			if !slugs.is_empty() { eprintln!(); }
		}
	}
	
	// Create output directory if not exists
	files::create_output_dir(&output_config.dir)?;
	
	// Download all recipes in list
	save_recipes(&url_config, &download_config, &output_config, &slugs);
	
	Ok(())
}


pub fn get_config(url_options: URLOptions, download_options: DownloadOptions) -> Result<(URLConfig, DownloadConfig), String> {
	let url_config = URLConfig {
		build_id: unwrap_or_get_build_id(url_options.build_id)?,
		username: url_options.username
	};
	let download_config = DownloadConfig {
		interval: Duration::from_millis(download_options.interval as u64)
	};
	Ok((url_config, download_config))
}

pub fn unwrap_or_get_build_id(build_id: Option<String>) -> Result<String, String> {
	if let Some(build_id) = build_id {
		return Ok(build_id);
	}
	eprintln!("No build ID specified, scraping current value from main site ...");
	
	let build_id = get_build_id()
		.map_err(|e| format!("Failed to get build ID: {e}"))?;
	eprintln!("Build ID: {build_id}");
	
	Ok(build_id)
}


// - Download

enum DownloadError {
	OutputFileAlreadyExists(PathBuf),
	RequestFailed(String),
	OutputFileWriteFailed(PathBuf, io::Error),
}

pub fn save_recipes(url_config: &URLConfig, download_config: &DownloadConfig, output_config: &OutputConfig, slugs: &[String]) -> Vec<(PathBuf, bool)> {
	let mut output_files: Vec<(PathBuf, bool)> = Vec::new();
	
	for slug in slugs {
		// Try downloading recipe
		utils::print_item(slug);
		let start = Instant::now();
		let result = save_recipe(url_config, output_config, slug);
		
		// Handle result
		match result {
			Ok(path) => {
				utils::print_item_success();
				output_files.push((path, true));
			},
			Err(e) => {
				match e {
					DownloadError::OutputFileAlreadyExists(path) => {
						utils::print_item_error("Skipped, output file already exists");
						output_files.push((path, false));
						continue; // Skip delay
					}
					DownloadError::RequestFailed(reason) => {
						utils::print_item_error(&reason);
					},
					DownloadError::OutputFileWriteFailed(path, e) => {
						utils::print_item_error(&format!("Could not write output file {}: {e}", path.display()));
					}
				}
			}
		}
		
		// Wait for next interval
		let delay = download_config.interval.saturating_sub(start.elapsed());
		if !delay.is_zero() {
			sleep(delay);
		}
	}
	
	output_files
}

fn save_recipe(url_config: &URLConfig, output_config: &OutputConfig, slug: &str) -> Result<PathBuf, DownloadError> {
	let filename = format!("{slug}.json");
	
	// Check if output file exists
	let output_file = output_config.dir.join(&filename);
	if !output_config.overwrite && output_file.exists() {
		return Err(DownloadError::OutputFileAlreadyExists(output_file));
	}
	
	// Get file contents
	let contents = get_user_data(url_config, &filename)
		.map_err(|e| DownloadError::RequestFailed(e))?;
	
	// Write output file
	fs::write(&output_file, contents)
		.map_err(|e| DownloadError::OutputFileWriteFailed(output_file.clone(), e))?;
	
	Ok(output_file)
}


// - Provecho.co Requests

pub fn get_all_recipe_slugs(config: &URLConfig) -> Result<Vec<String>, String> {
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
