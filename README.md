Screenshot Text Indexer
=======================

A small summer-holiday project to hack together some utilities for indexing and searching through
a collection of a large collection of screenshot images, using OCR to scrape the text from the
often text-heavy screenshots (e.g. web page contents, window titles, UI labels, etc.)

All this is designed to run fully offline / locally, on Windows 10 machines. Support for performing
the indexing on Linux (using a different OCR engine) is left as an exercise for the user.

## Design Philosophy

Get something working ASAP for my own needs, that works on my machine (and hopefully others too),
and have some fun building it.

## Utilities

* **`py_indexer`** - A Python 3 script that uses the Windows 10 built-in OCR Engine to perform
  OCR on all the images that it finds in a directory, writing all the scraped text to a massive
  JSON file ("text_index / database") for later processing.

* **`rust_search_cli`** - A Rust-based tool for querying the database produced by `py_indexer`,
  allowing some initial quick searches of the database to identify files of interest. This is
  mainly to test that our approach for loading / deserialising the database works, and to have
  something to do some initial testing with.

* **`rust_search_ui`** - A Rust-based UI tool for searching the database, with image previews,
  search-match overlays, and advanced tools for performing region-based filtering. For example:
  1) Extracting only the text in a certain area (for extracting values from standardised UI forms), OR
  2) Excluding all the text from certain areas from search results (e.g. for excluding browser quick-link
     toolbar labels from the results, to prevent false-positives)

## License

* Free to use / modify / adapt for personal + small business task-automation use.

* Not for use by government agencies and/or firms involved in intelligence gathering,
  ad-targetting, surveillance, espionage, and/or similar activities.

* Use for training, running, or testing AI services (including but not limited to text-generation
  services such as Co-Pilot, ChatGPT, or flavour-of-the-month auto-complete) is generally NOT allowed.
  
  An exception is made however for use in the development of improved OCR engines and/or language
  translation services (but only to natural human languages. Translations to machine-language /
  programming-language to circumvent the No AI restrictions are NOT allowed).
 
* No warranty / fitness for purpose / or liabilities are implied or claimed. The author(s) are not
  to be held liable or responsible for any consequences arising from this code.


