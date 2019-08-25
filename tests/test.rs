extern crate inline_assets;

#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};

#[cfg(test)]
use insta::assert_debug_snapshot_matches;

#[test]
fn basic() {
    assert_debug_snapshot_matches!(
        "listener_screen_basic",
		inline_assets::inline_file(
			"examples/resources/listener_screen.html",
			inline_assets::Config {
				remove_new_lines: false,
				inline_fonts: false
			}
		)
		.unwrap()
	);
}

#[test]
fn css_import_and_http() {
	assert_debug_snapshot_matches!(
		"listener_screen_css_import",
		inline_assets::inline_file(
			"examples/resources/listener_screen_css_import.html",
			inline_assets::Config {
				remove_new_lines: false,
				inline_fonts: false,
			}
		)
		.unwrap()
	);
}

//#[test]
//fn remove_new_lines() {
//	assert_eq!(
//		inline_assets::inline_file(
//			"examples/resources/listener_screen_css_import.html",
//			Default::default()
//		)
//		.unwrap(),
//		include_str!("listener_remove_new_lines.compiled.html")
//	);
//}
