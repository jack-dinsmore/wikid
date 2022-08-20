use crate::constants::*;
use clap::{ArgMatches};
use crate::root::Root;
use std::fs::{remove_dir_all, create_dir};
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
    
    let public = matches.is_present("public");
    if public {
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
    let compile_tree = Node::new()?;
    println!("Compiling {} files", compile_tree.size());
    let ref_map = compile_tree.ref_map(public)?;

    compile_tree.compile(&mut file_queue, &ref_map, public)?;


    // Write
    let mut target_existed = true;
    let target_dir = Root::get_path_from_local("html/")?;
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
        let path = root.get_link_from_local("html/index.html", public)?;
        if let Err(_) = open::that(&path) {
            return Err(format!("Could not display website. Link searched was {}", path));
        }
    }

    Ok(())
}