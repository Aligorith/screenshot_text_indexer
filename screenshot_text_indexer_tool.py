# Screenshot Text Indexer Tool
import sys
import os

import asyncio
import json
import pathlib

from PIL import Image
import winocr

###########################################
# Constants

# Help text for when no arguments are provided
# TODO: Include optional args...
USAGE_TEXT = """\
screenshot_text_indexer_tool.py [...root/folder/path/...]
"""

# Supported file formats
SUPPORTED_FILE_FORMATS = {
	'.png',
	'.jpg',
	'.jpeg',
	#'.gif',
	'.tif',
	'.tiff',
	'.webm',
	'.bmp',
}

###########################################
# Output Database Abstraction

# Base-class for output database objects
class ResultDatabaseBaseclass:
	# Store results-dict for a particular filename
	# < key: (str) Filepath of the image that the contents came from
	# < data: (JSON-dict) Dictionary containing the results of run_ocr_on_image()
	def add_result(self, key: str, data: dict):
		pass
	
	# Ensure all results have been flushed to the backing file...
	def flush(self):
		pass

# -----------------------------------------

# JSON File database Backend
class JsonResultDatabase(ResultDatabaseBaseclass):
	# ctor
	def __init__(self,
	             db_fileN: str,
	             progressive_saves=True):
		# Add extension to the given filename + store it
		self.db_fileN = db_fileN + ".json"
		
		# Cache of the final dict to write...
		self._data = {}
		
		# Flush as we go>
		# (Enabled by default to avoid data loss)
		self.progressive_saves = progressive_saves
		
	# Store results for file to the database
	# :: ResultDatabaseBaseClass.add_result()
	def add_result(self, key: str, data: dict):
		# Warn if we have duplicate keys, in case of data loss...
		if key in self._data:
			print("WARNING: File path already in results dict - '%s'" % (key),
			      filo=sys.stderr)
		
		# Save the data to the dict
		self._data[key] = data
		
		# Flush, if doing progressive saves...
		if self.progressive_saves:
			self.flush()
	
	# Ensure all results have been flushed to the backing file...
	def flush(self):
		with open(self.db_fileN, 'w') as f:
			json_data_str = json.dumps(self._data, indent='\t')
			f.write(json_data_str)

# -----------------------------------------

# SQL (sqlite) Database Backend
class SqlResultDatabase(ResultDatabaseBaseclass):
	pass

###########################################
# Image Processor

# < fileN: (str) Filepath of image to extra text from
# < lang: (str | [str]) String ID for language-processing backend to use
#                       or list of languages to use.
#
# More info about the languages supported:
# https://learn.microsoft.com/en-us/windows/uwp/app-resources/how-rms-matches-lang-tags         
def run_ocr_on_image(fileN: str, lang='en'):
	try:
		img = Image.open(fileN)
	except IOError as err:
		print("ERROR: Could not load '%s' as image for processing..." % (fileN))
		return {}
	
	# For multiple languages, construct a dict for each language...
	# FIXME: `windows.storage.streams.DataWriter :: .write_bytes()` crashes
	#        when passing it data from full-sized screenshots...
	if type(lang) is str:
		print("processing '%s'" % fileN)
		result = winocr.recognize_pil_sync(img, lang)
		print("file done...")
	else:
		result = {
			lang_key : winocr.recognize_pil_sync(img, lang_key)
			for lang_key in lang
		}
	
	return result

###########################################
# Directory Walker

# Generator function that yields the images to be processed
# >> yields: Tuples of the form: `(root_folder, fileN, file_path)` for each file,
#            where `file_path == root_folder + fileN`
#
# TODO: Yield progress indicator too?
def file_walker(root_path: str, quiet=False):
	# Walk the directory tree...
	for (this_root_path, folders, files) in os.walk(root_path):
		if not quiet:
			print("Checking folder '%s' - %d files..." % (this_root_path, len(files)))
		
		# Iterate over this folder's files
		for fileN in files:
			# Only continue if this is potentially an image file...
			# FIXME: Move past whitelisting image formats like this
			extn = os.path.splitext(fileN)[1].lower()
			if extn not in SUPPORTED_FILE_FORMATS:
				print("skipping '%s'" % fileN)
				continue;
			
			# Construct a file-path for this filename
			file_path = os.path.join(this_root_path, fileN)
			yield (this_root_path, fileN, file_path)

###########################################
# Entrypoint

def main():
	args = sys.argv[1:]
	
	db_format = "json"
	#db_format = "sql"
	
	# TODO: Process args properly...
	if len(args) > 0:
		# Assume first is a path...
		root_path = args[0]
	else:
		# Use current folder
		print(USAGE_TEXT)
		root_path = "."   # XXX: Expand?
	
	
	# Prepare database
	db_fileN = os.path.join(root_path, "extracted_text_index")
	
	print("> Opening output database for writing - '%s.%s'" % (db_fileN, db_format))
	if db_format == 'json':
		db = JsonResultDatabase(db_fileN)
	elif db_format == 'sql':
		db = SqlResultDatabase(db_fileN)
	else:
		print("ERROR: Unsupported database format ('%s'). Aborting.",
		      file=sys.stderr)
		sys.exit(1)  ## EARLY ABORT
	
	print("DB Filename = '%s'" % (db.db_fileN))
	
	# Walk that directory tree, processing the images...
	print("\n> Extracting text from images in folder...")
	for (base_path, filename, filepath) in file_walker(root_path):
		# Process file...
		# TODO: Skip processing on files already in the DB unless the hash signature changed
		result = run_ocr_on_image(filepath)
		db.add_result(filepath, result)
	
	# Do a final flush
	print("\n> Processing done. Saving results to file...")
	db.flush()

# -----------------------------------------

if __name__ == "__main__":
	main()
