use clap::Parser;
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use std::io::Write;
use std::fs::{self, File, OpenOptions};
use crate::constants::*;
use crate::root::Root;

#[derive(Parser)]
pub struct AddSettings {
    /// Name of the section to add
    name: String,
    /// Color of the section
    #[arg(long)]
    color: String, 
}

#[derive(Parser)]
pub struct RmSettings {
    /// Name of the section to remove
    name: String,
    /// Set to true to force removal
    #[arg(long)]
    force: bool,
}

#[derive(Parser)]
pub struct MvSettings {
    /// Name of the section to move
    name: String,
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub color: String,
    pub ignore: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveSection {
    pub color: String,
    pub ignore: bool,
}

impl Section {
    pub fn new(name: String, color: Color) -> MyResult<Section> {
        if let Err(_) = fs::create_dir(&name) {
            return Err("Could not create section directory.".to_owned());
        }
        if let Err(_) = File::create(format!("{}/_toc.md", name)) {
            return Err("Could not create section table of contents.".to_owned());
        }

        let out = Section {
            name: name.to_owned(),
            color: color.to_string(),
            ignore: false
        };

        // Save
        let json_text = match serde_json::to_string(&out.save_section()) {
            Err(_) => return Err("Failed to write root data to json".to_owned()),
            Ok(t) => t
        };
        let mut file = match File::create(format!("{}/.wikid.json", &name)) {
            Ok(f) => f,
            Err(_) => return Err("Could not create wikid.json".to_owned())
        };
        if let Err(_) = file.write_all(json_text.as_bytes()) {
            return Err("Could not write to wikid.json".to_owned());
        }

        Ok(out)
    }

    fn save_section(&self) -> SaveSection {
        SaveSection {
            color: self.color.clone(),
            ignore: self.ignore,
        }
    }

    pub(crate) fn from_save_section(sec: SaveSection, name: &str) -> Section {
        Self {
            name: name.to_owned(),
            color: sec.color.clone(),
            ignore: sec.ignore,
        }
    }
}


impl AddSettings {
    pub fn run(&self) -> MyResult<()> {
        let color = match Color::from_str(self.color.trim_end()) {
            Ok(c) => c,
            Err(_) => return Err("Please give a proper hex-formatted color (e.g., #abcdef).".to_owned())
        };
    
        Section::new(self.name.clone(), color)?;
    
        // Add to the toc
        {
            let mut file = match OpenOptions::new()
            .write(true)
            .append(true)
            .open(Root::get_path_from_local("text/_toc.md")?) {
                Ok(f) => f,
                Err(_) => return Err("Could not open table of contents".to_owned())
            };
    
            if let Err(_) = file.write_all(&format!("* [{0}]{{{0}/_toc}}\n", self.name).as_bytes()) {
                return Err("Could not add section name to toc.".to_owned());
            }
        }
    
        println!("Created section");
    
        Ok(())
    }
}

// impl RmSettings {
//     pub fn run(&self) -> MyResult<()> {
//         let files = match fs::read_dir(Root::get_path_from_local(&format!("text/{}", self.name))?) {
//             Ok(p) => p,
//             Err(_) => return Err("That section did not exist".to_owned())
//         };

//         // Check if there are files in the section
//         if !self.force {
//             let mut directory_is_full = false;
//             for file in files {
//                 let file_name = match file {
//                     Ok(f) => f.file_name(),
//                     Err(_) => return Err("Could not read all the files in the section directory".to_owned())
//                 };
//                 if file_name != "_toc.md" {
//                     println!("Encountered file {:?} in section {}", file_name, self.name);
//                     directory_is_full = true;
//                 }
//             }
//             if directory_is_full {
//                 return Err("Cannot delete a full directory without a force".to_owned())
//             }
//         }

//         // Delete the section
//         let mut root = Root::summon()?;

//         if let Err(_) = remove_dir_all(Root::get_path_from_local(&format!("text/{}", self.name))?) {
//             return Err("Cannot remove the section file".to_owned());
//         };
//         root.write()?;

//         println!("Section successfully removed");

//         Ok(())
//     }
// }

// pub fn list_sections() -> MyResult<()> {
//     for sec in Root::summon()?.sections {
//         println!("{}", sec.name);
//     }
//     Ok(())
// }
