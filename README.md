Screenshot Text Indexer
=======================

Python 3 tool that runs on Windows 10 (and perhaps Windows 11 - untested)
for extracting the text from screenshots, making it easier to manage and
search through them.

It works by going over all the image files in a nominated directory (using the
current directory if nothing is provided), and/or recursing into any
sub-folders found, and will run the Windows built-in OCR engine on
those images, extracting any text found.

It has several modes of operation:
1) Dump extracted-text index to JSON file
2) Dump extracted-text index to sqlite database


## Dependencies

* Windows 10+
* Python 3.7+
* Windows SDK PIP Library (i.e. "winsdk")

## Language Support

The OCR engine only supports the OCR Language Packs you have installed on your machine
when running this software.

For instructions on installing these, see:
* https://docs.uipath.com/studio/docs/installing-ocr-languages#section-installing-an-ocr-engine-and-changing-the-language

