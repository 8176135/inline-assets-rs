extern crate inline_assets;
//#[macro_use]
//extern crate pretty_assertions;

#[test]
fn basic() {
    assert_eq!(inline_assets::inline_file("examples/resources/listener_screen.html", inline_assets::Config { remove_new_lines: false, inline_fonts: true }).unwrap(), include_str!("listener_screen.compiled.html"));
}

#[test]
fn css_import_and_http() {
    assert_eq!(inline_assets::inline_file("examples/resources/listener_screen_css_import.html", inline_assets::Config { remove_new_lines: false, inline_fonts: true }).unwrap(), include_str!("listener_screen_css_import.compiled.html"));
}

#[test]
fn remove_new_lines() {
    assert_eq!(inline_assets::inline_file("examples/resources/listener_screen_css_import.html", Default::default()).unwrap(), include_str!("listener_remove_new_lines.compiled.html"));
}