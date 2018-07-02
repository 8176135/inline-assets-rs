inline-assets-rs
=====
[![Build Status](https://travis-ci.org/Hand-of-Cthulhu/inline-assets-rs.svg?branch=master)](https://travis-ci.org/Hand-of-Cthulhu/inline-assets-rs)

A Rust library for inlining Javascript, CSS, and font files into your html files for easy distribution.
This also changes `url(local_path)` in the css to be relative to the html file.

Originally intended to be used in build scripts of [web_view](https://github.com/Boscop/web-view "Rust bindings to zserge/webview") projects.
The output could also be able to be piped into [minifier-rs](https://github.com/GuillaumeGomez/minifier-rs) to save space.

### Usage:
Usage is really simple, just call `inline_html_string(file_path, inline_fonts)`, with the html file path,
 and whether you want to embed the fonts as base64 in the css.
 
Look in the example folder for a CLI binary example.

All font files should work if font format is set correctly. i.e.`src: url(font-file) format(font-format)`

To import CSS recursively, use `@import url(path_to_another_css_file);` in your css files. Multiple imports of the same CSS file will only be imported once.

### TODO:
* Support inlining images