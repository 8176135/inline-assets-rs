extern crate regex;
extern crate base64;

use std::fs;
use std::io::ErrorKind as IoErrorKind;
use std::path::Path;
use regex::Captures;

/// Augmented std::io::Error so that it shows what line is causing the problem.
#[derive(Debug)]
pub enum FilePathError {
    /// A std::io::ErrorKind::NotFound error with the offending line in the string parameter
    InvalidPath(String),
    /// Any other file read error that is not NotFound
    FileReadError(String, std::io::Error),
}

impl std::error::Error for FilePathError {
    fn description(&self) -> &str {
        &match *self {
            FilePathError::InvalidPath(_) => "Invalid path, file not found",
            FilePathError::FileReadError(_,_) => "Error during file reading"
        }
    }
}

impl std::fmt::Display for FilePathError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            FilePathError::InvalidPath(ref line) => write!(f, "Invalid path: {}", line),
            FilePathError::FileReadError(ref cause, ref io_err) => write!(f, "Cause: {}, File read error: {}", cause, io_err)
        }
    }
}

impl FilePathError {
    fn from_elem(e: std::io::Error, elem: &str) -> Self {
        match e.kind() {
            IoErrorKind::NotFound => FilePathError::InvalidPath(format!("File not found: {}", elem)),
            _ => FilePathError::FileReadError(elem.to_owned(), e)
        }
    }
}

impl From<std::io::Error> for FilePathError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            IoErrorKind::NotFound => FilePathError::InvalidPath("File not found".to_owned()),
            _ => FilePathError::FileReadError("N/A".to_owned(),e)
        }
    }
}

/// Returns a `Result<String, FilePathError>` of the html file at file path with all the assets inlined.
///
/// ## Arguments
/// * `file_path` - The path of the html file.
/// * `inline_fonts` - Whether or not to inline fonts in the css as base64.
pub fn inline_file<P: AsRef<Path>>(file_path: P, inline_fonts: bool) -> Result<String, FilePathError> {
    let html = fs::read_to_string(&file_path).map_err(|orig_err| FilePathError::from_elem(orig_err, "Html file not found"))?;
    inline_html_string(&html, &file_path.as_ref().parent().unwrap(), inline_fonts)
}

/// Returns a `Result<String, FilePathError>` with all the assets linked in the the html string inlined.
///
/// ## Arguments
/// * `html` - The html string.
/// * `root_path` - The root all relative paths in the html will be evaluated with, usually this is the folder the html file is in.
/// * `inline_fonts` - Whether or not to inline fonts in the css as base64.
///
pub fn inline_html_string<P: AsRef<Path>>(html: &str, root_path: P, inline_fonts: bool) -> Result<String, FilePathError> {
    let root_path = root_path.as_ref();

    let link_finder = regex::Regex::new(r#"<link[^>]+?href\s*=\s*['"]([^"']+)['"][^>]*?>"#).unwrap(); // Finds css <link href="path"> tags
    let script_finder = regex::Regex::new(r#"<script[^>]+?src\s*=\s*['"]([^"']+?)['"][^>]*?>\s*?</\s*?script\s*?>"#).unwrap(); // Finds javascript <script src="path"></script> tags
    let font_url_finder = regex::Regex::new(r#"(@font-face[\s]*?\{[^}]*?src:[\s]*?url\()("?[^()'"]+?"?)(\))"#).unwrap(); // Finds @font-face { src: url(path) } in the css
    let url_finder = regex::Regex::new(r#"url\s*?\(([^"')]+?)\)"#).unwrap(); // Finds all url(path) in the css and makes them relative to the html file

    let mut is_alright: Result<(), FilePathError> = Ok(());
    let out_html = link_finder.replace_all(html, |caps: &Captures| {
        let css_path = root_path.join(&caps[1]);

        let css_data = match fs::read_to_string(&css_path) {
            Ok(css_data) => url_finder.replace(css_data.as_str(), |caps: &Captures|
                format!("url({})", css_path.parent().unwrap().join(&caps[1]).to_str().expect("Path not UTF-8")
                    .replace("\\", "/"))).to_string(),
            Err(e) => {
                is_alright = Err(FilePathError::from_elem(e, &caps[0]));
                return "Error".to_owned();
            }
        };
        if is_alright.is_err() {
            return "".to_owned();
        }
        let css_data = if inline_fonts {
            font_url_finder.replace_all(css_data.as_str(), |caps: &Captures| {
                match fs::read(&caps[2]) {
                    Ok(font_data) => format!("${}data:application/font-woff;charset=utf-8;base64,{}{}", &caps[1], base64::encode(&font_data), &caps[3]),
                    Err(e) => {
                        is_alright = Err(FilePathError::from_elem(e, &caps[0]));
                        return "Error".to_owned();
                    }
                }
            }).to_string()
        } else {
            css_data
        };

        format!("<style>{}</style>", css_data)
    }).to_string();

    if is_alright.is_err() {
        return Err(is_alright.unwrap_err());
    }


    // TODO: Support type tags in output
    let out_html = script_finder.replace_all(out_html.as_str(), |caps: &Captures| {
        format!("<script>{}</script>", match fs::read_to_string(root_path.join(&caps[1])) {
            Ok(res) => res,
            Err(e) => {
                is_alright = Err(FilePathError::from_elem(e, &caps[0]));
                return "Error".to_owned();
            }
        })
    }).to_string();

    if is_alright.is_err() {
        return Err(is_alright.unwrap_err());
    }

    Ok(out_html)
}
