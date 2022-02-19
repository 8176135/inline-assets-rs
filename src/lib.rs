extern crate base64;
extern crate kuchiki;
extern crate regex;
#[macro_use]
extern crate html5ever;

mod test;

use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind as IoErrorKind;
use std::path::{Path, PathBuf};

use kuchiki::traits::TendrilSink;
use kuchiki::NodeRef;
use regex::Captures;
use std::str::FromStr;

/// Augmented std::io::Error so that it shows what line is causing the problem.
#[derive(Debug)]
pub enum FilePathError {
	/// A std::io::ErrorKind::NotFound error with the offending line in the string parameter
	InvalidPath(String),
	/// Any other file read error that is not NotFound
	FileReadError(String, std::io::Error),
	/// A css file is imported twice, or there is a dependency loop
	RepeatedFile,
}

/// Config struct that is passed to `inline_file()` and `inline_html_string()`
///
/// Default enables everything
#[derive(Debug, Copy, Clone)]
pub struct Config {
	/// Whether or not to inline fonts in the css as base64.
	pub inline_fonts: bool,
	/// Replace `\r` and `\r\n` with a space character. Useful to keep line numbers the same in the output to help with debugging.
	pub remove_new_lines: bool,
}

impl Default for Config {
	/// Enables everything
	fn default() -> Config {
		Config {
			inline_fonts: true,
			remove_new_lines: true,
		}
	}
}

impl std::error::Error for FilePathError {
	fn description(&self) -> &str {
		&match *self {
			FilePathError::InvalidPath(_) => "Invalid path, file not found",
			FilePathError::FileReadError(_, _) => "Error during file reading",
			FilePathError::RepeatedFile => {
				"File is imported twice, or there is a circular dependency"
			}
		}
	}
}

impl std::fmt::Display for FilePathError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			FilePathError::InvalidPath(ref line) => write!(f, "Invalid path: {}", line),
			FilePathError::FileReadError(ref cause, ref io_err) => {
				write!(f, "Cause: {}, File read error: {}", cause, io_err)
			}
			FilePathError::RepeatedFile => write!(
				f,
				"A file is imported twice, or there is a circular dependency"
			),
		}
	}
}

impl FilePathError {
	fn from_elem(e: std::io::Error, elem: &str) -> Self {
		match e.kind() {
			IoErrorKind::NotFound => {
				FilePathError::InvalidPath(format!("File not found: {}", elem))
			}
			_ => FilePathError::FileReadError(elem.to_owned(), e),
		}
	}
}

impl From<std::io::Error> for FilePathError {
	fn from(e: std::io::Error) -> Self {
		match e.kind() {
			IoErrorKind::NotFound => FilePathError::InvalidPath("File not found".to_owned()),
			_ => FilePathError::FileReadError("N/A".to_owned(), e),
		}
	}
}

/// Returns a `Result<String, FilePathError>` of the html file at file path with all the assets inlined.
///
/// ## Arguments
/// * `file_path` - The path of the html file.
/// * `inline_fonts` - Pass a config file to select what features to enable. Use `Default::default()` to enable everything
pub fn inline_file<P: AsRef<Path>>(file_path: P, config: Config) -> Result<String, FilePathError> {
	let html = fs::read_to_string(&file_path)
		.map_err(|orig_err| FilePathError::from_elem(orig_err, "Html file not found"))?;
	inline_html_string(&html, &file_path.as_ref().parent().unwrap(), config)
}

/// Returns a `Result<String, FilePathError>` with all the assets linked in the the html string inlined.
///
/// ## Arguments
/// * `html` - The html string.
/// * `root_path` - The root all relative paths in the html will be evaluated with, usually this is the folder the html file is in.
/// * `config` - Pass a config file to select what features to enable. Use `Default::default()` to enable everything
///
pub fn inline_html_string<P: AsRef<Path>>(
	html: &str,
	root_path: P,
	config: Config,
) -> Result<String, FilePathError> {
	// FIXME: make actual error return
	let root_path = root_path.as_ref().canonicalize().unwrap();
	let document = kuchiki::parse_html().one(html);
	
	let mut css_path_set = HashSet::new();

	let mut to_delete_vec = Vec::new();

	for css_match in document.select("script, link").unwrap() {
		let as_node = css_match.as_node();

		let node = as_node.as_element().unwrap();

		match node.name.local.to_string().as_str() {
			"script" => {
				let mut text_attr = node.attributes.borrow_mut();
				if let Some(c) = text_attr.get("src") {
					let sub_script_path = PathBuf::from_str(c).expect("script src not valid path");
					let script_path = if sub_script_path.is_absolute() {
						sub_script_path
					} else {
						root_path.join(sub_script_path)
					};

					text_attr.remove("src");
					as_node.append(NodeRef::new_text(
						fs::read_to_string(&script_path).map_err(|e| {
							FilePathError::from_elem(e, &script_path.to_string_lossy().to_string())
						})?,
					));
				} else {
					continue;
				}
			}
			"link" => {
				let css_path = {
					let mut text_attr = node.attributes.borrow_mut();
					let out = if let Some(c) = text_attr
						.get("rel")
						.filter(|rel| *rel == "stylesheet")
						.and(text_attr.get("href"))
					{
						root_path.join(PathBuf::from_str(c).expect("href not valid path"))
					} else {
						continue;
					};
					out
				};

				let css = inline_css(css_path, &root_path, &mut css_path_set)
					.expect("Failed to inline css");

				let elem_to_add = NodeRef::new_element(
					html5ever::QualName::new(None, ns!(html), "style".into()),
					None,
				);

				elem_to_add.append(NodeRef::new_text(css));
				as_node.insert_after(elem_to_add);
				to_delete_vec.push(css_match);
			}
			_ => {}
		}
	}

	for css_match in to_delete_vec {
		css_match.as_node().detach();
	}

	let answer = document.to_string()
	// Use new unix style newline
		.replace("\r\n", "\n");
	
	Ok(if config.remove_new_lines {
		answer.replace("\r\n", " ").replace("\n", " ")
	} else {
		answer
	})
}

fn inline_css<P: AsRef<Path>, P2: AsRef<Path>>(
	css_path: P,
	root_path: P2,
	path_set: &mut HashSet<std::path::PathBuf>,
) -> Result<String, FilePathError> {
	let css_path = css_path
		.as_ref()
		.canonicalize()
		.map_err(|e| FilePathError::from_elem(e, css_path.as_ref().to_str().unwrap()))?;
	if !path_set.insert(css_path.clone()) {
		return Err(FilePathError::RepeatedFile);
	}

	//	let css_data = fs::read_to_string(&css_path)
	//		.map_err(|e| FilePathError::from_elem(e, css_path.to_str().unwrap()))?;

	let comment_remover = regex::Regex::new(r#"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/"#).unwrap();

	// Some optimisation could be done here if we don't initialize these every single time.
	let css_finder: regex::Regex =
		regex::Regex::new(r#"@import[\s]+url\(["']?([^"']+)["']?\)\s*;"#).unwrap(); // Finds all @import url(style.css)
	let url_finder = regex::Regex::new(r#"url\s*?\(["']?([^"')]+?)["']?\)"#).unwrap(); // Finds all url(path) in the css and makes them relative to the html file

	let mut is_alright: Result<(), FilePathError> = Ok(());
	let css_data = css_finder
		.replace_all(
			url_finder
				.replace_all(
					comment_remover
						.replace_all(
							&fs::read_to_string(&css_path).map_err(|e| {
								FilePathError::from_elem(e, css_path.to_str().unwrap())
							})?,
							|_: &Captures| "".to_owned(),
						)
						.as_ref(),
					|caps: &Captures| {
						if caps[1].len() > 1500 || caps[1].contains("data:") {
							// Probably not a path if longer than 1500 characters
							return caps[0].to_owned();
						}
						format!(
							"url({})",
							if (caps[1].as_ref() as &str).contains("://") {
								caps[1].to_owned()
							} else {
								pathdiff::diff_paths(
									css_path.parent().unwrap().join(&caps[1]).as_path(),
									root_path.as_ref(),
								)
								.unwrap()
								.as_path()
								.to_str()
								.expect("Path not UTF-8")
								.replace("\\", "/")
							}
						)
					},
				)
				.as_ref(),
			|caps: &Captures| {
				match inline_css(
					root_path.as_ref().join(&caps[1]),
					root_path.as_ref(),
					path_set,
				) {
					Ok(out) => out,
					Err(FilePathError::RepeatedFile) => {
						"".to_owned() // Ignore repeated file
					}
					Err(e) => {
						is_alright = Err(e);
						return "Error".to_owned();
					}
				}
			},
		)
		.to_string();

	if is_alright.is_err() {
		return Err(is_alright.unwrap_err());
	}

	Ok(css_data)
}
