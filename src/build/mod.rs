use crate::constants::*;
use crate::root::Root;
use std::fs::{remove_dir_all, create_dir};
use crate::build::file_queue::FileQueue;
use crate::build::compile_tree::Node;

mod css;
mod refs;
mod file_queue;
mod compile_tree;
mod compile;
mod links;

use clap::Parser;
use css::build_css;

#[derive(Parser)]
pub(crate) struct BuildSettings {
    #[arg(short, long)]
    /// Compile with public links
    public: bool,
    #[arg(short, long)]
    /// Open HTML after build
    run: bool,
    /// Verbosity
    #[arg(long, short)]
    verbose: bool,
}

impl BuildSettings {
    pub fn run(&self) -> MyResult<()> {
        unsafe {
            crate::VERBOSE = self.verbose;
        }
        let root = Root::summon()?;
        
        if self.public {
            println!("Public build");
            if root.public_url == "" {
                return Err("You must first set the public url".to_owned());
            }
        }
        
        let mut file_queue = FileQueue::new();

        if let Some(image) = &root.bg_image {
            if !image.ends_with(".png") {
                return Err(format!("The background image {} is not a PNG", image));
            }
            //TODO do this
            file_queue.append_imgs(vec![(image.to_owned(), "html/css/background_image.png".to_owned())]);
        }
        
        // Make css files
        if crate::is_verbose() {
            println!("Building css files");
        }
        build_css(&root, &mut file_queue);
        
        // Compile
        let compile_tree = Node::new()?;
        if crate::is_verbose() {
            println!("Compiling {} files", compile_tree.size());
        }
        let ref_map = compile_tree.ref_map(self.public)?;
        
        compile_tree.compile(&mut file_queue, &ref_map, self.public)?;
        
        
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
        if crate::is_verbose() {
            println!("Writing {} files", file_queue.size());
        }
        file_queue.write(&target_dir)?;
        
        if crate::is_verbose() {
            println!("Succeeded");
        }
        
        if self.run {
            let path = root.get_link_from_local("html/index.html", self.public)?;
            if let Err(_) = open::that(&path) {
                return Err(format!("Could not display website. Link searched was {}", path));
            }
        }
        
        Ok(())
    }
}