# pdfiron

<p align="center">
  <img src="misc/icon.png">
</p>



Basically a part-reimplementation of [pdfsandwich](http://www.tobias-elze.de/pdfsandwich/index.html). Thus it uses [unpaper](https://github.com/unpaper/unpaper) to optimize the documents and [ŧesseract](https://github.com/tesseract-ocr/tesseract) for OCR. Motivation for this project:

- Pdfiron supports the splitting of double layout pages (two pages per sheet) into two individual output pages.
- The execution of tesseract is optional.

Pdfiron makes full usage of multi core systems and distributes the work of each step on as many cores as available on the system.

