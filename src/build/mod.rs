use crate::constants::*;
use clap::{ArgMatches};
use crate::root::Root;
use std::fs::{remove_dir_all, create_dir, read_dir, File};
use crate::build::file_queue::FileQueue;
use crate::build::compile_tree::Node;

mod css;
mod refs;
mod file_queue;
mod compile_tree;
mod compile;

use css::build_css;

pub fn build<'a>(matches: &ArgMatches<'a>) -> MyResult<()> {
    let root = Root::summon()?;

    if matches.is_present("public") {
        println!("Public build");
        if root.public_url == "" {
            return Err("You must first set the public url".to_owned());
        }
    }

    let mut file_queue = FileQueue::new();

    // Make css files
    println!("Building css files");
    build_css(&root, &mut file_queue);

    // Compile
    let compile_tree = Node::new();
    println!("Compiling {} files", compile_tree.size());
    let ref_map = compile_tree.ref_map(&root, matches.is_present("public"))?;
    compile_tree.compile(&mut file_queue, &ref_map, matches.is_present("public"))?;


    // Write
    let mut target_existed = true;
    let target_dir = Root::concat_root_dir("html/")?;
    if let Err(_) = remove_dir_all(&target_dir) {
        target_existed = false;
    }
    if let Err(_) = create_dir(&target_dir) {
        if !target_existed {
            return Err("Could not create target directory".to_owned());
        }
        else {
            return Err("Could not clean target directory".to_owned());
        }
    }

    println!("Writing {} files", file_queue.size());
    file_queue.write(&target_dir)?;

    println!("Succeeded");

    if matches.is_present("run") {
        println!("{}", root.public_url);
        let path = if matches.is_present("public") {
            println!("Public url was {}", root.public_url);
            unimplemented!();
        } else {
            format!("file://{}", Root::concat_root_dir("html/index.html")?)
        };
        if let Err(_) = open::that(&path) {
            return Err(format!("Could not display website. Link searched was {}", path));
        }
    }

    Ok(())
}