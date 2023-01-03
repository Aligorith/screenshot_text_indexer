#[macro_use] extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate prompts;


use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;

//use std::time::{Duration, Instant};
use std::time::Instant;

//use serde::{Deserialize, Serialize};
//use serde_json::Result;

use futures::executor::block_on;
use prompts::{Prompt, text::TextPrompt};

// =====================================================
// Constants

// Dump the short-form of the entries identified...
const DEBUG_ENTRIES: bool = false;

// =====================================================
// Data Structures 

// XXX: Is it possible to unpack the root-level hashmap into this nested structure?
// e.g. using `deserialize_map<V>(self, visitor: V)`, where map has <String, ImageTextInfo> pairs ?

// Bounding box for each word
#[derive(Serialize, Deserialize)]
struct ImageWordBBox {
	height: f32,
	width: f32,
	x: f32,
	y: f32,
}

// Each individual "word" that was identified
#[derive(Serialize, Deserialize)]
struct ImageTextWord {
	// Bounding box of the word
	bounding_rect: ImageWordBBox,
	
	// The word's text
	text: String
}


// Line / Sentence Fragment found in the image
#[derive(Serialize, Deserialize)]
struct ImageTextLine {
	// A string representing the sentence that the words in this line spell out
	text: String,
	
	// Each individual "word" that was identified
	words: Vec<ImageTextWord>,
}


// Root-Level entry corresponding to each image that was indexed...
#[derive(Serialize, Deserialize)]
struct ImageTextInfo {
	// A list of lines / sentence-fragments of text found in the image
	lines: Vec<ImageTextLine>,
	
	// A string containing all the text extracted from the image
	text: String,
}

// Alias for top-level data-structure
type ImagesTextIndex = HashMap<String, ImageTextInfo>;

// =====================================================
// Search Logic

// Search the index for a term - Returns a list of filenames that contain this term...
fn find_term(text_index: &ImagesTextIndex,
             search_term: &str)
            -> Vec<String>
{
	// TODO: Return the match too?
	let mut matching_filenames: Vec<String> = Vec::new();
	
	// Search over the index
	// TODO: Optimise this, by multi-threading?
	for (key, info) in text_index.iter() {
		// TODO: Fuzzy search?
		
		// NOTE: Use lowercase version of string to avoid need for strict case matching
		let info_text = info.text.to_lowercase();
		if info_text.contains(search_term) {
			matching_filenames.push(String::from(key));
		}
	}
	
	// Perform natural sort on these, so they appear in an organised way,
	// instead of in random hash/thread-traversal order
	matching_filenames.sort_unstable_by(|a, b| {
		lexical_sort::natural_lexical_cmp(&a, &b)
	});
	
	return matching_filenames;
}

// =====================================================

// Load index from file
fn load_index_from_file(filename: &str) -> ImagesTextIndex
{
	println!("Loading index from '{}'...", filename);
	let loading_timer = Instant::now();
	
	// Load the given JSON file
	let json_str = fs::read_to_string(filename).expect("Unable to read index database file");
	
	// Try to deserialise it to a HashMap of struts
	// TODO: Split all this loading code into a helper function
	let text_index : ImagesTextIndex = 
		serde_json::from_str(&json_str).expect("Could not unpack JSON");
	
	let loading_duration = loading_timer.elapsed();
	println!("Index Loaded in {:?}.  {} entries found.\n", loading_duration, text_index.len());
	
	// Return (move) the index to the parent
	return text_index;
}

// -----------------------------------------------------

// Dump the list of entries...
fn print_entries(text_index: &ImagesTextIndex,
                 max_entries: Option<usize>)
{
	// Print header text
	match max_entries {
		Some(x) => println!("\nFirst {} Entries:", x),
		None    => println!("\nEntries:")
	}
	
	// FIXME: Arms have different types, if we try to do this the most natural way...
	let entries_iter = match max_entries {
		Some(x) => {
			// Only first n
			text_index.iter().take(x).into_iter()
		},
		None => {
			// Include all
			//text_index.iter()  // XXX: We cannot reconcile the types this way, hence the hack below
			text_index.iter().take(text_index.len())
		}
	};
	
	// Print the first n entries...
	for (key, image_info) in entries_iter {
		println!(">>  '{}':",  key);
		println!("    '{}'\n", image_info.text);
	}
}

// Alias for print_entries(), without needing to pass `None` to get all the entries
fn print_all_entries(text_index: &ImagesTextIndex)
{
	print_entries(text_index, None);
}

// -----------------------------------------------------

// Search REPL loop - Needs to be in a function,
// as the async stuff can't be in main()
async fn search_repl_loop(text_index: &ImagesTextIndex)
{
	println!("Enter a term to search for (fuzzy search), or <Ctrl-C> to exit:");
	let mut prompt = TextPrompt::new(">> ");
	match prompt.run().await {
		Ok(Some(s)) => {
			// Handle "empty text" case...
			if s == "" {
				// Print the entries log - only the first n ones though...
				print_entries(&text_index, Some(10));
				println!("\n\n");
				return;
			}
			
			// TODO: Perform fuzzy search...
			let search_timer = Instant::now();
			let results = find_term(&text_index, &s);
			let search_op_duration = search_timer.elapsed();
			
			// Format the vector as a string
			let results_string = results.join("\n");
			
			// Print these results on screen
			println!("\nFound matches in {} images in {:?}:", results.len(), search_op_duration);
			println!("{}", results_string);
			println!("\n");
			
			// Also dump these results in a file for further processing...
			fs::write("./last_matching_files.txt", results_string).expect("Unable to write file");
		},
		
		Ok(None) => {
			println!("Ctrl-C pressed. Exiting.");
			process::exit(0);
		},
		
		Err(e) => println!("Some kind of crossterm error happened: {:?}", e),
	}
}


// -----------------------------------------------------

fn main()
{
	let args: Vec<String> = env::args().collect();
	
	if let Some(db_filename) = args.get(1) {
		let text_index = load_index_from_file(db_filename);
		
		// Debug: Print the entries found in the database...
		if DEBUG_ENTRIES {
			println!("\nEntries:");
			for (key, image_info) in text_index.iter() {
				println!(">>  '{}':\n    '{}'\n", key, image_info.text);
			}
			print_all_entries(&text_index);
		}
		
		
		// Search Query Prompt loop...
		// Main thread is blocked while this thing runs...
		loop {
			let repl_loop_result = search_repl_loop(&text_index);
			block_on(repl_loop_result);
		}
	}
	else {
		println!("USAGE: screenshot_search_tool <json_index_db.json>");
		process::exit(1);  // Exit with 'dirty' status...
	}
}
