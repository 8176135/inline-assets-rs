extern crate inline_assets;
extern crate clap;

fn main() {
    let matches = clap::App::new("HTML_Inliner")
        .version("0.1")
        .author("hand-of-cthulhu")
        .arg(clap::Arg::with_name("embed-font")
            .short("f")
            .long("embed-font")
            .help("Embeds fonts as base64 in css"))
        .arg(clap::Arg::with_name("html")
            .help("HTML file path")
            .index(1).required(true)).get_matches();

    println!("{}", inline_assets::inline_file(matches.value_of("html").unwrap(), inline_assets::Config { inline_fonts: matches.occurrences_of("embed-font") == 1, ..Default::default()}).unwrap());
}