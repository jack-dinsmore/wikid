use clap::ArgMatches;
use serde::{Serialize, Deserialize};
use std::str::{FromStr};
use std::io::stdin;
use std::fs::{self, File, remove_dir_all};
use crate::constants::*;
use crate::root::Root;

#[derive(Serialize, Deserialize, Debug)]
pub struct Section {
    pub name: String,
    pub color: String,
}

impl Section {
    pub fn new(name: String, color: Color) -> Section {
        Section { name, color: color.to_string() }
    }
}

pub fn add_section(matches: &ArgMatches) -> MyResult<()> {
    println!("What color would you like the section to be?");

    let name = matches.value_of("new").expect("NEW_NAME was required");

    let color = loop {
        let mut buffer = String::new();
        if let Err(_) = stdin().read_line(&mut buffer) {
            return Err("Could not read terminal buffer");
        }
        match Color::from_str(buffer.trim_end()) {
            Ok(c) => break c,
            Err(_) => println!("Please give a proper hex-formatted color (e.g., #abcdef;).")
        }
    };

    let mut root = Root::summon()?;
    for sec in &root.sections {
        if sec.name == name {
            return Err("This name has already been taken");
        }
    }

    let sec = Section::new(name.to_string(), color);
    root.sections.push(sec);

    root.write()?;

    let mut section_path = Root::concat_root_dir("text/")?;
    section_path.push_str(name);

    if let Err(_) = fs::create_dir(&section_path) {
        return Err("Could not create section directory.");
    }


    if let Err(_) = File::create(format!("{}/_toc.md", section_path)) {
        return Err("Could not create section table of contents.");
    }

    println!("Created section");

    Ok(())
}

pub fn delete_section(matches: &ArgMatches) -> MyResult<()> {
    let name = matches.value_of("delete").expect("DELETE_NAME was required");

    let files = match fs::read_dir(Root::concat_root_dir(&format!("text/{}", name))?) {
        Ok(p) => p,
        Err(_) => return Err("That section did not exist")
    };

    let forced = matches.is_present("force");

    // Check if there are files in the section
    if !forced {
        let mut directory_is_full = false;
        for file in files {
            let file_name = match file {
                Ok(f) => f.file_name(),
                Err(_) => return Err("Could not read all the files in the section directory")
            };
            if file_name != "_toc.md" {
                println!("Encountered file {:?} in section {}", file_name, name);
                directory_is_full = true;
            }
        }
        if directory_is_full {
            return Err("Cannot delete a full directory without a force")
        }
    }

    // Delete the section
    if let Err(_) = remove_dir_all(format!("text/{}", name)) {
        return Err("Cannot remove the section file");
    };
    let mut root = Root::summon()?;
    let index = root.sections.iter().position(|x| x.name == name).unwrap();
    root.sections.remove(index);
    root.write()?;

    println!("Section successfully removed");

    Ok(())
}

pub fn list_sections(matches: &ArgMatches) -> MyResult<()> {
    for sec in Root::summon()?.sections {
        println!("{}", sec.name);
    }
    Ok(())
}
