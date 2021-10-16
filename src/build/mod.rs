use crate::constants::*;
use clap::{ArgMatches};
use crate::root::Root;
use std::str::FromStr;
use std::fs::{remove_dir_all, create_dir};
use crate::build::refs::RefMap;
use crate::build::file_queue::FileQueue;

mod css;
mod refs;
mod file_queue;

use css::build_css;

pub fn build<'a>(matches: &ArgMatches<'a>) -> MyResult<()> {
    let root = Root::summon()?;

    if matches.is_present("public") {
        println!("Public build");
        if root.public_url == "" {
            return Err("You must first set the public url");
        }
    }

    let mut file_queue = FileQueue::new();

    build_css(&root, &mut file_queue);
    let ref_map = RefMap::new(&root);
    
    // Write
    let mut target_existed = true;
    let target_dir = Root::concat_root_dir("target/")?;
    if let Err(_) = remove_dir_all(&target_dir) {
        target_existed = false;
    }
    if let Err(_) = create_dir(&target_dir) {
        if !target_existed {
            return Err("Could not create target directory");
        }
        else {
            return Err("Could not clean target directory");
        }
    }

    file_queue.write(&target_dir)?;

    Ok(())
}
