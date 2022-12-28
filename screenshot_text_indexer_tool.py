# Screenshot Text Indexer Tool
import sys
import os

import asyncio
import json
import pathlib
import time

from winsdk.windows.media.ocr import OcrEngine
from winsdk.windows.globalization import Language
from winsdk.windows.storage import FileAccessMode, StorageFile
from winsdk.windows.graphics.imaging import BitmapDecoder, BitmapPixelFormat

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
	'.webp',
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
# TODO: Write multiple versions - one with full (raw) data, and one with text only?
class JsonResultDatabase(ResultDatabaseBaseclass):
	# ctor
	def __init__(self, db_fileN: str):
		# Add extension to the given filename + store it
		self.db_fileN = db_fileN + ".json"
		
		# Cache of the final dict to write...
		self._data = {}
		
	# Store results for file to the database
	# :: ResultDatabaseBaseClass.add_result()
	def add_result(self, key: str, data: dict):
		# Warn if we have duplicate keys, in case of data loss...
		if key in self._data:
			print("WARNING: File path already in results dict - '%s'" % (key),
			      filo=sys.stderr)
		
		# Save the data to the dict
		self._data[key] = data
	
	# Ensure all results have been flushed to the backing file...
	def flush(self):
		with open(self.db_fileN, 'w') as f:
			json.dump(self._data, f, indent='\t')

# -----------------------------------------

# SQL (sqlite) Database Backend
class SqlResultDatabase(ResultDatabaseBaseclass):
	pass

###########################################
# Image Processor

# Helper Utilities (from winocr) ----------

# Needed to get asyncio to run the async code to be runnable...
async def await_coroutine_result(awaitable):
	return await awaitable

# Convert wrapped Windows-SDK return objects as Python dicts,
# to make them easier to work with...
#
# From winocr.py
def picklify(o):
	if hasattr(o, 'size'):
		return [picklify(e) for e in o]
	elif hasattr(o, '__module__'):
		return dict([(n, picklify(getattr(o, n))) for n in dir(o) if not n.startswith('_')])
	else:
		return o

# -----------------------------------------

# < fileN: (str) Filepath of image to extra text from
# < lang: (str) String ID for language-processing backend to use
#
# More info about the languages supported:
# https://learn.microsoft.com/en-us/windows/uwp/app-resources/how-rms-matches-lang-tags         
def run_ocr_on_image(fileN: str, lang='en'):
	# Adapted from `https://github.com/8/ConsoleUwpOcr -> "Program.cs"`
	async def _run_winsdk_ops():
		filePath = os.path.abspath(fileN)   # winsdk needs absolute paths only...
		
		file = await StorageFile.get_file_from_path_async(filePath);
		stream = await file.open_async(FileAccessMode.READ);
		decoder = await BitmapDecoder.create_async(stream);
		softwareBitmap = await decoder.get_software_bitmap_async();
		
		# TODO: Modify this part to create a dict that contains multiple language results...
		engine = OcrEngine.try_create_from_language(Language(lang))
		return await engine.recognize_async(softwareBitmap)
	
	
	# Run the helper method, which runs asynchronously...
	result_obj = asyncio.run(await_coroutine_result( _run_winsdk_ops() ))
	
	# Turn it into a plain Python dict, so we can dump the data for debugging/saving
	# much easier...
	result = picklify(result_obj)
	
	#print(result_obj.text)
	#pprint(result)
	
	# Return the result dict
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
	# TODO: Switch to multi-processing approach to be faster?
	print("\n> Extracting text from images in folder...")
	
	start_time = time.time()
	i = 0
	
	for (base_path, filename, filepath) in file_walker(root_path):
		# Report progress occasionally...
		if (i > 0) and (i % 500 == 0):
			elapsed_time = time.time() - start_time
			print("... %d files processed so far, after %.3f sec" % (i, elapsed_time))
			
			db.flush()   # flush so we don't lose everything...
		
		# Process file...
		# TODO: Skip processing on files already in the DB unless the hash signature changed
		# TODO: Skip file if it cannot be found now (i.e. it was moved before the processing happened)
		result = run_ocr_on_image(filepath)
		db.add_result(filepath, result)
		
		# Increment progress counter...
		i += 1
	
	# Do a final flush
	print("\n> Processing done. Saving results to file...")
	db.flush()
	
	# Report total time taken
	end_time = time.time()
	time_taken_sec = end_time - start_time
	print("\n>> Time Taken = %.3f sec" % (time_taken_sec))

# -----------------------------------------

if __name__ == "__main__":
	main()
