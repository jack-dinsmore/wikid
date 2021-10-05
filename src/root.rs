use std::fs;
use clap::{ArgMatches};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::*;
use crate::constants::*;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    wikid_version_major: u32,
    wikid_version_minor: u32,
    name: String,
}

impl Root {
    fn new(name: String) -> Root {
        Root { wikid_version_major: WIKID_VERSION_MAJOR, wikid_version_minor: WIKID_VERSION_MINOR,
            name}
    }

    fn get_root_dir() -> MyResult<PathBuf> {
        // Scan for the wikid file.
        let mut path = Path::new("./");
        loop {
            let paths = match fs::read_dir("./") {
                Ok(p) => p,
                Err(_) => return Err("Wikid has not been initialized in this directory.")
            };
            for test_path in paths {
                let path = match test_path {
                    Ok(p) => p.path(),
                    Err(e) => continue
                };
                if path.is_dir() {
                    if let Some(n) = path.file_name() {
                        if n == ".wikid" {
                            return Ok(path);
                        }
                    }
                }
            }
            path = match path.parent() {
                Some(p) => p,
                None => return Err("Wikid has not been initialized in this directory.")
            };
        }
    }

    pub fn summon() -> MyResult<Root> {
        let paths = match fs::read_dir(Root::get_root_dir()?) {
            Ok(p) => p,
            Err(_) => return Err("Wikid directory was invalid")
        };

        for test_path in paths {
            let path = match test_path {
                Ok(p) => p.path(),
                Err(e) => continue
            };
            if !path.is_dir() {
                if let Some(n) = path.file_name() {
                    if n == "wikid.json" {
                        return Ok(match serde_json::from_str(&match fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(_) => return Err("Could not open wikid.json")
                        }) {
                            Ok(r) => r,
                            Err(_) => return Err("Wikid.json was corrupted")
                        });
                    }
                }
            }
        }
        Err("Could not find the wikid.json file.")
    }
}

pub fn init(matches: &ArgMatches) -> MyResult<()> {
    if let Err(_) = fs::create_dir(".wikid") {
        return Err("Could not create .wikid directory");
    }

    let root = Root::new(matches.value_of("name")
                                .expect("name is a required argument")
                                .to_string());

    let json_text = match serde_json::to_string(&root) {
        Err(e) => return Err("Failed to write root data to json"),
        Ok(t) => t
    };

    let mut file = match File::create(".wikid/wikid.json") {
        Ok(f) => f,
        Err(e) => return Err("Could not create wikid.json")
    };
    if let Err(e) = file.write_all(json_text.as_bytes()) {
        return Err("Could not write to wikid.json");
    }

    Ok(())
}
