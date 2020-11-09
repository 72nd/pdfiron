# pdfiron

<p align="center">
  <img width="500" src="misc/logo.png">
</p>

Pdfiron is basically a part-reimplementation of [pdfsandwich](http://www.tobias-elze.de/pdfsandwich/index.html). Thus it uses [unpaper](https://github.com/unpaper/unpaper) to optimize the documents and [ŧesseract](https://github.com/tesseract-ocr/tesseract) for OCR. The main motivation was to support the splitting of double layouts (two book pages per scanned page in the input file) into individual pages in the output document. It's also possible to skip Tesseract and produce optimized documents faster.


## Motivation and Example

Studying human sciences at university demands reading lots of text. Frustratingly these are often provided as nearly unreadable scans. Further on this documents contain almost always only an image of the scanned pages, thus it's not possible to copy any content out of this files. This tool combines a number of different applications to provide readable PDF's with embedded text (via OCR).

For an example, take this scan from Friedrich Nietzsches [Also sprach Zarathustra](https://en.wikipedia.org/wiki/Also_sprach_Zarathustra):

![Example Scan Zarathustra](misc/example-1.png)

## Technical details


- Pdfiron supports the splitting of double layout pages (two pages per sheet) into two individual output pages.
- The execution of tesseract is optional.

Pdfiron makes full usage of multi core systems and distributes the work of each step on as many cores as available on the system.

## Todo's

- [ ] Rotate input PDF's before processing.
- [ ] Ignore some pages bug
- [ ] Readme
	- [ ] Example
	- [ ] Usage 
	- [ ] Installation
	- [ ] Skip Tesseract
