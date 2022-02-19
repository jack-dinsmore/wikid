use clap::ArgMatches;
use serde::{Serialize, Deserialize};
use std::str::{FromStr};
use std::io::{stdin, Write};
use std::fs::{self, File, remove_dir_all, OpenOptions};
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
            return Err("Could not read terminal buffer".to_owned());
        }
        match Color::from_str(buffer.trim_end()) {
            Ok(c) => break c,
            Err(_) => println!("Please give a proper hex-formatted color (e.g., #abcdef;).")
        }
    };

    let mut root = Root::summon()?;
    for sec in &root.sections {
        if sec.name == name {
            return Err("This name has already been taken".to_owned());
        }
    }

    let sec = Section::new(name.to_string(), color);
    root.sections.push(sec);
    let mut section_path = Root::get_path_from_local("text/")?;
    section_path.push_str(name);

    if let Err(_) = fs::create_dir(&section_path) {
        return Err("Could not create section directory.".to_owned());
    }
    if let Err(_) = File::create(format!("{}/_toc.md", section_path)) {
        return Err("Could not create section table of contents.".to_owned());
    }

    // Add to the toc
    {
        let mut file = match OpenOptions::new()
        .write(true)
        .append(true)
        .open(Root::get_path_from_local("text/_toc.md")?) {
            Ok(f) => f,
            Err(_) => return Err("Could not open table of contents".to_owned())
        };

        if let Err(_) = file.write_all(&format!("* [{0}]{{{0}/_toc}}\n", name).as_bytes()) {
            return Err("Could not add section name to toc.".to_owned());
        }
    }

    // Write changes 
    if let Err(e) = root.write() {
        root.sections.pop();
        return Err(e);
    }

    println!("Created section");

    Ok(())
}

pub fn delete_section(matches: &ArgMatches) -> MyResult<()> {
    let name = matches.value_of("delete").expect("DELETE_NAME was required");

    let files = match fs::read_dir(Root::get_path_from_local(&format!("text/{}", name))?) {
        Ok(p) => p,
        Err(_) => return Err("That section did not exist".to_owned())
    };

    let forced = matches.is_present("force");

    // Check if there are files in the section
    if !forced {
        let mut directory_is_full = false;
        for file in files {
            let file_name = match file {
                Ok(f) => f.file_name(),
                Err(_) => return Err("Could not read all the files in the section directory".to_owned())
            };
            if file_name != "_toc.md" {
                println!("Encountered file {:?} in section {}", file_name, name);
                directory_is_full = true;
            }
        }
        if directory_is_full {
            return Err("Cannot delete a full directory without a force".to_owned())
        }
    }

    // Delete the section
    let mut root = Root::summon()?;

    if let Err(_) = remove_dir_all(Root::get_path_from_local(&format!("text/{}", name))?) {
        return Err("Cannot remove the section file".to_owned());
    };
    let index = root.sections.iter().position(|x| x.name == name).unwrap();
    root.sections.remove(index);
    root.write()?;

    println!("Section successfully removed");

    Ok(())
}

pub fn list_sections() -> MyResult<()> {
    for sec in Root::summon()?.sections {
        println!("{}", sec.name);
    }
    Ok(())
}
