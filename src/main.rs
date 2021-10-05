// (Full example with detailed comments in examples/01b_quick_example.rs)
//
// This example demonstrates clap's full 'builder pattern' style of creating arguments which is
// more verbose, but allows easier editing, and at times more advanced options, or the possibility
// to generate arguments dynamically.
extern crate clap;
use clap::{Arg, App, SubCommand};
mod root;
mod constants;
mod build;
use root::init;
use crate::constants::*;
use build::build;

fn main() {
    let version_text = format!("{}.{}", WIKID_VERSION_MAJOR, WIKID_VERSION_MINOR);
    let matches = App::new("Wikid")
        .version(&version_text[..])
        .author("Jack Dinsmore <jtdinsmo@mit.edu>")
        .about("Compiles wiki and blog posts into HTML")
        .subcommand(SubCommand::with_name("init")
            .about("Initializes a wiki")
            .arg(Arg::with_name("name")
                .help("Name to give the wiki")
                .required(true)))
        .subcommand(SubCommand::with_name("build")
            .about("Compile markdown into html"))
        .get_matches();

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    if let Some(matches) = matches.subcommand_matches("init") {
        if let Err(msg) = init(matches) {
            println!("Init failed: {}", msg);
        }
    }
    else if let Some(matches) = matches.subcommand_matches("build") {
        if let Err(msg) = build(matches) {
            println!("Build failed: {}", msg);
        }
    }
    else{
        println!("You must provide a valid subcommand");
    }
}
