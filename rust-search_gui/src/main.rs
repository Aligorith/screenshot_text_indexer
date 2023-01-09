#[macro_use] extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

use iced::{
	Color, Command, Element, Font, Length, Sandbox,
	Settings, Subscription,
};
use iced::alignment::{self, Alignment};
use iced::event::{self, Event};
use iced::keyboard;
use iced::subscription;
use iced::theme::{self, Theme};
use iced::widget::{
	self,
	button, column, container, image, row,
	scrollable, text, text_input, tooltip,
};
use iced::window;

use log::debug;

use once_cell::sync::Lazy;


// =====================================================
// Image Text-Index Types
// Note: Copied "rust-search_cli"
//       TODO: De-duplicate these by splitting these defs into a shared crate?

// Bounding box for each word
#[derive(Serialize, Deserialize, Debug)]
struct ImageWordBBox {
	height: f32,
	width: f32,
	x: f32,
	y: f32,
}

// Each individual "word" that was identified
#[derive(Serialize, Deserialize, Debug)]
struct ImageTextWord {
	// Bounding box of the word
	bounding_rect: ImageWordBBox,
	
	// The word's text
	text: String
}


// Line / Sentence Fragment found in the image
#[derive(Serialize, Deserialize, Debug)]
struct ImageTextLine {
	// A string representing the sentence that the words in this line spell out
	text: String,
	
	// Each individual "word" that was identified
	words: Vec<ImageTextWord>,
}


// Root-Level entry corresponding to each image that was indexed...
#[derive(Serialize, Deserialize, Debug)]
struct ImageTextInfo {
	// A list of lines / sentence-fragments of text found in the image
	lines: Vec<ImageTextLine>,
	
	// A string containing all the text extracted from the image
	text: String,
}


// Alias for top-level data-structure
type ImagesTextIndex = HashMap<String, ImageTextInfo>;

// =====================================================
// Image Text-Index Methods
// TODO: Split to separate module / deduplicate with the above type defs

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
// Iced UI - Utilities

fn empty_message(message: &str) -> Element<'_, Message>
{
	container(
		text(message)
			.width(Length::Fill)
			.size(25)
			.horizontal_alignment(alignment::Horizontal::Center)
			.style(Color::from([0.7, 0.7, 0.7])),
	)
	.width(Length::Fill)
	.height(Length::Units(200))
	.center_y()
	.into()
}

// =====================================================
// Iced UI - Search Match Items

// Search Match Item Model
#[derive(Debug, Clone)]
struct SearchMatchImage {
	// Filename / Filepath from the DB of the image matching the query
	filename : String,
	
	// Reference to the corresponding ImageTextInfo object
	// XXX: Beware lifetimes!!!
	
	// Image handle - for drawing to iced::widget::image::Viewer
	image_handle: image::Handle,
}

// TODO: Create SearchMatchImage's from list of matching filenames from the DB

impl SearchMatchImage {
	// Generate view delegate for visualising this item in the list view
	fn view(&self, i: usize) -> Element<Message>
	{
		row!(
			text(&self.filename)
		).into()
	}
}

// =====================================================
// Iced UI - Main App

// Application State
#[derive(Debug, Default)]
struct SearchGui {
	// Current Application State ------------------------
	
	app_state: SearchGuiState,
	
	// Images Text Index Database -----------------------
	
	// The filename of the database
	db_filename : PathBuf,
	
	// The image index loaded from the db
	text_index : ImagesTextIndex,
	
	// Search Options -----------------------------------
	
	// The string to search for
	search_text : String,
	
	// TODO: Search options - e.g. case sensitivity, fuzzy search
	
	// TODO: Regions to only consider matches from
	// TODO: Regions to ignore matches from
	// XXX: What to do if those two overlap?
	
	// Search Matches -----------------------------------
	
	// List of matching files from the DB matching the filter
	search_matches : Vec<SearchMatchImage>,
	
	// Currently selected match index
	current_idx : Option<usize>,
	
	// Currently selected filepath
	current_match: Option<SearchMatchImage>,
}

// ======================================================

// Current Application State
#[derive(Debug, Clone)]
enum SearchGuiState {
	// No index db loaded - prompt the user to select + load one
	FindIndexDb,
	
	// Show a "loading" message when trying to parse an index database	
	LoadingIndex,
	
	// Show the searching UI
	SearchTask,
}

impl Default for SearchGuiState {
    fn default() -> Self
    {
    	SearchGuiState::FindIndexDb
        //SearchGuiState::SearchTask
    }
}

// ------------------------------------------------------

// Application Messages
#[derive(Debug, Clone)]
//#[derive(Debug, Clone, Copy)]   // XXX: `Copy` not available for String
enum Message {
	// Show file picker dialog to select an index to load
	LoadExistingDbPicker,
	
	// Text in "search_text" box changed
	// - Flush the search string to state...
	SearchTextChanged(String),
	
	// Used to trigger a search operation
	TriggerSearch,
	
	// Search Match Ready - Used to trigger the UI to refresh to show the
	// newly arrived search results
	SearchResultsUpdated,
	
	// Some event (e.g. selection) occurred on the item with the given index
	SearchMatchMessage(usize),
}

// ======================================================

// ID for search box input box
static SEARCH_BOX_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);


// Application / Controller - Standard Framework Extension Points
impl Sandbox for SearchGui {
	type Message = Message;
	
	fn new() -> Self
	{
		// TODO: Work the loading into this?
		Self {
			// Default state...
			app_state: SearchGuiState::FindIndexDb,
			//app_state: SearchGuiState::LoadingIndex,
			//app_state: SearchGuiState::SearchTask,
			
			.. Self::default()
		}
	}

	fn title(&self) -> String 
	{
		// TODO: Include searching / operating states?
		let app_title = "Image Text DB Search";
		
		if self.search_matches.len() > 0 {
			format!("{} - {} matches", app_title, self.search_matches.len())
		}
		else {
			app_title.into()
		}
	}
	
	fn update(&mut self, message: Message)
	{
		match message {
			// [SearchGuiState::FindIndexDb] .............................
			Message::LoadExistingDbPicker => {
				// Show file picker - If successful, trigger next steps
				self.browse_for_index_path();
			}
			
			// [SearchGuiState::SearchTask] ..............................
			Message::SearchTextChanged(query_string) => {
				// Copy search text
				self.search_text = query_string.clone();
				// TODO: Trigger search...
			}
			Message::TriggerSearch => {
				// TODO: Launch async search
			}
			Message::SearchResultsUpdated => {
				// Reset current search match 
				// --> Default to nothing selected, as there may not be any matches
				//     FIXME: Maybe this isn't it... As Dynamic search will keep losing the current selection!
				self.current_idx = None;
				self.current_match = None;
			}
			Message::SearchMatchMessage(idx) => {
				// Update current index, and get the matching filepath from that index
				// TODO: Need to get the index from the message?
			}
		}
	}
	
	fn view(&self) -> Element<Message>
	{
		match self.app_state {
			SearchGuiState::FindIndexDb => {
				self.view_find_db_prompt()
			}
			SearchGuiState::LoadingIndex => {
				self.view_index_loading_ui()
			}
			SearchGuiState::SearchTask => {
				self.view_search_ui()	
			}
		}
	}
}


// Own event-handler function defines for SearchGui
impl SearchGui {
	// Show a file browser to pick an index file
	// Used for:  `Message::LoadExistingDbPicker` case
	fn browse_for_index_path(&mut self)
	{
		let start_directory = std::env::current_dir().unwrap_or_default();
		
		let file_dialog_result = rfd::FileDialog::new()
			.add_filter("All File Types", &["*"])  // XXX
			.set_directory(&start_directory)
			.pick_file();

		if let Some(file_path) = file_dialog_result {
			debug!("Selected File Path = {:?}", file_path);
			self.db_filename = file_path;
		}
	}
}

// Own view-function defines for SearchGui
impl SearchGui {
	// Prompt the user to load a database (or generate one)
	fn view_find_db_prompt(&self) -> Element<Message>
	{
		let prompt_text = 
			text("Select or generate a database of indexed text-snippets to search through:");
		
		let find_button =
			tooltip(
				button("Loading Existing...")
					//.size(14)
					.on_press(Message::LoadExistingDbPicker),
				"Load index from an existing image text index db file",
				tooltip::Position::Bottom
			)
			.style(theme::Container::Box);
		
		container(
			column![
				prompt_text,
				
				row![
					find_button,
					//generate_button,
				]
			]
			.align_items(Alignment::Center)
		)
		.center_y()
		.into()
	}
	
	// Loading screen to display while loading / parsing the index
	fn view_index_loading_ui(&self) -> Element<Message>
	{
		container(
			column![
				text("Loading index database...")
					.size(22),
				text(format!("\"{:?}\"", self.db_filename))
					.size(14),
			]
			.align_items(Alignment::Center)
		)
		.center_y()
		.into()
	}
	
	// Main UI for searching through the index
	fn view_search_ui(&self) -> Element<Message>
	{
		// LHS: Search Panel - Query String Box
		let search_box = text_input(
				"Text to search for...",
				&self.search_text,
				Message::SearchTextChanged
			)
			.id(SEARCH_BOX_ID.clone())
			.padding(15)
			//.size(12)
			.on_submit(Message::TriggerSearch);
		
		// LHS: Search Panel - Search Matches List
		let matches_box: Element<_> = 
			if !self.search_matches.is_empty() {
				// Populate search-match list with view delegates for each match
				// and put that in a scrollable
				scrollable(
					container(
						column(
							self.search_matches
								.iter()
								.enumerate()
								.map(|(i, img_match)| {
									img_match.view(i).map(move |message| {
										Message::SearchMatchMessage(i)
									})
								})
								.collect()
						)
						.spacing(5)
					)
					.width(Length::Fill)
					.padding(40)
					.center_x(),
				)
				.into()
			}
			else {
				// Placeholder
				empty_message("No matching images")
			};
		
		// LHS: Search Panel - Putting it all together
		let search_panel = column!(
				search_box,
				matches_box,
			)
			.width(Length::Units(300))
			.spacing(10);
		
		// RHS: Matching Image Panel
		// See pokedex
		// 	`img = image::Handle::from_path(url))`
		let image_panel: Element<_> = match &self.current_match {
			Some(current_match) => {
				// Image Viewer
				column!(
					image::viewer(current_match.image_handle.clone())
					// TODO: Markup toolbar?
				)
				.width(Length::Fill)
				.into()
			}
			None => {
				// Placeholder
				empty_message("Select an image match to see it here...")
			}
		};
		
		// Overall Layout - 2 panel horizontal split
		// TODO: Adjustable Splitter -> Use PaneGrid?
		// TODO: How to enable .explain()?
		row!(
			search_panel,
			image_panel,
		)
		.into()
	}
}



// =====================================================


fn main() -> iced::Result
{
	SearchGui::run(Settings::default())
}
