extern crate inline_assets;

#[test]
fn it_works() {
    assert_eq!(inline_assets::inline_file("examples/resources/listener_screen.html", true).unwrap(), include_str!("listener_screen.compiled.html"));
    assert_eq!(2 + 2, 4);
}
