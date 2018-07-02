inline-assets-rs
=====
A Rust library for inlining Javascript, CSS, and font files into your html files for easy distribution.
This also changes `url(path)` in the css to be relative to the html file

Originally intended to be used in build scripts of [web_view](https://github.com/Boscop/web-view "Rust bindings to zserge/webview") projects.
The output could also be able to be piped into [minifier-rs](https://github.com/GuillaumeGomez/minifier-rs) to save space.

### Usage:
Usage is really simple, just call `inline_html_string(file_path, inline_fonts)`, with the html file path,
 and whether you want to embed the fonts as base64 in the css.
 
Look in the example folder for a CLI binary example.