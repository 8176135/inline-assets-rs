extern crate inline_assets;

#[test]
fn basic() {
    assert_eq!(inline_assets::inline_file("examples/resources/listener_screen.html", true).unwrap(), include_str!("listener_screen.compiled.html"));
}
#[test]
fn css_import_and_http() {
    assert_eq!(inline_assets::inline_file("examples/resources/listener_screen_css_import.html", true).unwrap(), include_str!("listener_screen_css_import.compiled.html"));
}
