use std::fs;
use clap::{ArgMatches};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::*;
use crate::constants::*;
use std::path::{Path};
use git2::Repository;
use crate::section::Section;

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    wikid_version_major: u32,
    wikid_version_minor: u32,
    name: String,
    pub public_url: String,
    pub sections: Vec<Section>,
    pub main_contents_level: u8, // Deepest level of the main table of contents
}


impl Root {
    fn new(name: String) -> Root {
        Root { wikid_version_major: WIKID_VERSION_MAJOR,
            wikid_version_minor: WIKID_VERSION_MINOR,
            name,
            public_url: String::new(),
            sections: Vec::new(),
            main_contents_level: 2,
        }
    }

    pub fn get_root_dir() -> MyResult<String> {
        // Scan for the wikid file.
        let mut path = Path::new("./");
        loop {
            let paths = match fs::read_dir("./") {
                Ok(p) => p,
                Err(_) => return Err("Wikid has not been initialized in this directory.".to_owned())
            };
            for test_path in paths {
                let path = match test_path {
                    Ok(p) => p.path(),
                    Err(_) => continue
                };
                if path.is_dir() {
                    if let Some(n) = path.file_name() {
                        if n == ".wikid" {
                            let parent = match path.parent() {
                                Some(p) => p,
                                None => return Err(".wikid had no parent".to_owned())
                            };
                            return Ok(match parent.to_str(){
                                Some(s) => s.to_owned(),
                                None => return Err("Could not resolve .wikid path".to_owned())
                            });
                        }
                    }
                }
            }
            path = match path.parent() {
                Some(p) => p,
                None => return Err("Wikid has not been initialized in this directory.".to_owned())
            };
        }
    }

    pub fn concat_root_dir(right: &str) -> MyResult<String> {
        let mut root_dir = Root::get_root_dir()?;
        root_dir.push_str("/");
        root_dir.push_str(right);
        Ok(root_dir)
    }

    pub fn summon() -> MyResult<Root> {
        let paths = match fs::read_dir(Root::concat_root_dir(".wikid")?) {
            Ok(p) => p,
            Err(_) => return Err("Wikid directory was invalid".to_owned())
        };

        for test_path in paths {
            let path = match test_path {
                Ok(p) => p.path(),
                Err(_) => continue
            };
            if !path.is_dir() {
                if let Some(n) = path.file_name() {
                    if n == "wikid.json" {
                        return Ok(match serde_json::from_str(&match fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(_) => return Err("Could not open wikid.json".to_owned())
                        }) {
                            Ok(r) => r,
                            Err(_) => return Err("Wikid.json was corrupted".to_owned())
                        });
                    }
                }
            }
        }
        Err("Could not find the wikid.json file.".to_owned())
    }

    pub fn write(&self) -> MyResult<()> {
        let json_text = match serde_json::to_string(self) {
            Err(_) => return Err("Failed to write root data to json".to_owned()),
            Ok(t) => t
        };

        let mut file = match File::create(Root::concat_root_dir(".wikid/wikid.json")?) {
            Ok(f) => f,
            Err(_) => return Err("Could not create wikid.json".to_owned())
        };
        if let Err(_) = file.write_all(json_text.as_bytes()) {
            return Err("Could not write to wikid.json".to_owned());
        }

        Ok(())
    }

    pub fn get_github_url() -> MyResult<String> {
        let repo = match Repository::open(Root::get_root_dir()?) {
            Ok(r) => r,
            Err(_) => return Err("Could not find repo in this wiki".to_owned())
        };
        let origin = match repo.find_remote("origin") {
            Ok(o) => o,
            Err(_) => return Err("Could not find remote named origin".to_owned())
        };
        Ok(match origin.url() {
            Some(u) => u,
            None => return Err("Origin did not have a url".to_owned())
        }.to_string())
    }
}

pub fn init(matches: &ArgMatches) -> MyResult<()> {
    let name = matches.value_of("name")
                                .expect("name is a required argument")
                                .to_string();
    if let Err(_) = fs::create_dir(".wikid") {
        return Err("Could not create .wikid directory".to_owned());
    }

    if let Err(_) = fs::create_dir("code") {
        return Err("Could not create code directory".to_owned());
    }

    if let Err(_) = fs::create_dir("text") {
        return Err("Could not create text directory".to_owned());
    }

    if let Err(_) = fs::create_dir("html") {
        return Err("Could not create target directory".to_owned());
    }

    if let Err(_) = File::create("_toc.md") {
        return Err("Could not create table of contents".to_owned());
    }

    if !matches.is_present("nogit") {
        if let Err(_) = Repository::init(".") {
            return Err("Could not initialize github repository in this directory".to_owned());
        }
    }
    else {
        println!("Creating wiki without github integration");
    }
    println!("Created wiki {}", name);

    let root = Root::new(name);

    root.write()
}
