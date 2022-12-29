Screenshot Text Indexer
=======================

A small summer-holiday project to hack together some utilities for indexing and searching through
a few large collections of screenshots, using OCR to scrape the text from the often
text-heavy images (e.g. articles, social media posts, window titles, UI labels, etc.)

All this is designed to run fully offline / locally, on Windows 10 machines. Support for performing
the indexing on Linux (using a different OCR engine) is left as an exercise for the user.


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


## Performance Notes

On the main folder where this tool was intended to be used:
* The folder had about 15,000 files
* The Python indexing script took around > 5200 seconds to process that folder (single threaded mode)
* The resulting index file was around 630 mb
* The `rust_search_cli` tool takes about 35 seconds to load + parse the index file


## Design Philosophy

Get something working ASAP for my own needs, that works on my machine (and hopefully others too),
and have some fun building it.

### Tech Stack FAQ's

The Windows 10 built-in OCR Engine was chosen for the following reasons:
* It is "freely" available on my workstation, and runs offline (without needing to upload images
  to some web host), at least from what I can tell.
  
* In tests on a few images, it did a good job at accurately extracting the text from the screenshots
  and ran quite quickly (i.e. almost instantly, at least from Microsoft's C#-based UWP sample app).
  
* The alternative free / open source engines I tried (e.g. Tessaract and GOCR in particular IIRC)
  were not able to cope with the images I needed them to process. Those two in particular appear to
  be rather old-school (80's / 90's libs) that only work on scanned PDF documents, and not so much on
  screenshots (and in particular, completely failing in cases where there is text overlaid over a photo)


Getting this API working however was not straightforward. Several prior attempts failed:
* **`https://github.com/8/ConsoleUwpOcr`** - Initial attempts to get this C#-based command-line "desktop" app
  resulted in failure, Despite multiple attempts, I couldn't figure out how to resolve the compiler errors
  about not being able to find the relevant `windows.media.ocr` API's, despite installing / reinstalling
  all the different versions of `UwpDesktop`, `UwpDesktop-Updated`, `Microsoft.Windows.SDK.Contracts`, and/or
  upgrading Visual Studio from 2017 to 2022.

* **[winocr](https://github.com/GitHub30/winocr)** - This Python / PIP package was initially promising,
  but kept silently crashing in `DataWriter.write_bytes()` when passing it an actual screenshot image
  (vs a small partial test screenshot snippet).

As a result, our Python script just uses the Windows SDK library / API directly, using code adapted from
the ConsoleUwpOcr code.



## License

* Free to use / modify / adapt for personal + small business task-automation use.

* NOT for use by government agencies, firms, and/or individuals involved in intelligence gathering,
  ad-targetting, surveillance, espionage, and/or similar activities - especially for those purposes.

* NOT for use for training, running, or testing AI services (including but not limited to text-generation
  services such as Co-Pilot, ChatGPT, or flavour-of-the-month auto-complete / content generation tools).
  
  An exception is made however for use in the development of improved OCR engines and/or language
  translation services (but only to natural human languages. Translations to machine-language /
  programming-language to circumvent the No AI restrictions are NOT allowed).
 
* NO warranty / fitness for purpose / or liabilities are implied or claimed. The author(s) are not
  to be held liable or responsible for any consequences arising from this code/software.
  
  Usage of this code/software implies that you accept and understand all the risks.


