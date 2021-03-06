inline-assets-rs
=====
[![Build Status](https://travis-ci.org/Hand-of-Cthulhu/inline-assets-rs.svg?branch=master)](https://travis-ci.org/Hand-of-Cthulhu/inline-assets-rs)
[![Crates.io](https://meritbadge.herokuapp.com/inline-assets)](https://crates.io/crates/inline-assets)
[![Docs.rs](https://docs.rs/inline_assets/badge.svg)](https://docs.rs/inline_assets/)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/Hand-of-Cthulhu/inline-assets-rs/master/LICENSE)

A Rust library for inlining Javascript, CSS, and font files into your html files for easy distribution.
This also changes `url(local_path)` in the css to be relative to the html file.

Originally intended to be used in build scripts of [web_view](https://github.com/Boscop/web-view "Rust bindings to zserge/webview") projects.
The output can also be able to be piped into [minifier-rs](https://github.com/GuillaumeGomez/minifier-rs) to save space.

### Usage:
Usage is really simple, just call `inline_assets::inline_html_string(file_path, inline_assets::Config::default())`, with the html file path. 

The config currently provides 2 options (with the default being both enabled):
* `inline_fonts`: Whether or not to inline fonts in the css as base64.
* `remove_new_lines`:  Replace `\r` and `\r\n` with a space character. Useful to keep line numbers the same in the output to help with debugging.

Look in the example folder for a CLI binary example.

All font files should work if font format is set correctly. i.e.`src: url(font-file) format(font-format)`

To import CSS recursively, use `@import url(path_to_another_css_file);` in your css files. Multiple imports of the same CSS file will only be imported once.

### TODO:
* Support inlining images