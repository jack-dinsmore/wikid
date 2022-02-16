use clap::ArgMatches;
use serde::{Serialize, Deserialize};
use std::str::{FromStr};
use std::io::{stdin, BufReader, BufRead, Write};
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

    root.write()?;

    // Add to the toc
    {
        let mut file = match OpenOptions::new()
        .write(true)
        .append(true)
        .open(Root::concat_root_dir("_toc.md")?) {
            Ok(f) => f,
            Err(_) => return Err("Could not open table of contents".to_owned())
        };

        if let Err(e) = file.write_all(&format!("* [{0}]{{0}}\n", name).as_bytes()) {
            return Err("Could not add section name to toc.".to_owned());
        }
    }


    let mut section_path = Root::concat_root_dir("text/")?;
    section_path.push_str(name);

    if let Err(_) = fs::create_dir(&section_path) {
        return Err("Could not create section directory.".to_owned());
    }


    if let Err(_) = File::create(format!("{}/_toc.md", section_path)) {
        return Err("Could not create section table of contents.".to_owned());
    }

    println!("Created section");

    Ok(())
}

pub fn delete_section(matches: &ArgMatches) -> MyResult<()> {
    let name = matches.value_of("delete").expect("DELETE_NAME was required");

    let files = match fs::read_dir(Root::concat_root_dir(&format!("{}", name))?) {
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

    {
        let mut new_toc = String::new();
        let reader = BufReader::new(match File::open(Root::concat_root_dir("_toc.md")?) {
            Ok(f) => f,
            Err(_) => return Err("Could not open the table of contents file".to_owned())
        });
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue
            };
            if !line.contains(&format!("[{}]", name)) {
                new_toc.push_str(&line);
                new_toc.push_str("\n");
            }
        }
        let mut toc = match File::create(Root::concat_root_dir("_toc.md")?) {
            Ok(f) => f,
            Err(_) => return Err("Could not open the table of contents".to_owned())
        };
        if let Err(_) = toc.write_all(&new_toc.as_bytes()) {
            return Err("Could not write to the table of contents file".to_owned());
        };
    }

    if let Err(_) = remove_dir_all(format!("{}", name)) {
        return Err("Cannot remove the section file".to_owned());
    };
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
