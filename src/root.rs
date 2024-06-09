use std::path::Path;
use std::{fs, env};
use clap::Parser;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::*;
use crate::constants::*;
use crate::section::Section;
use git2::Repository;

const DEFAULT_COLOR: &'static str = "#cccccc";

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    pub wikid_version_major: String,
    pub wikid_version_minor: String,
    pub name: String,
    pub public_url: String,
    pub bg_image: Option<String>,
    pub fonts: Option<Vec<String>>,
    pub main_color: String,
    pub dark_mode: bool,
    pub font_size: u32,
}

#[derive(Parser)]
pub struct InitSettings {
    /// Name of the wiki
    name: String,
    /// Set to turn of git init
    #[arg(long)]
    nogit: bool,
    /// Verbosity
    #[arg(long, short)]
    verbose: bool,
}

impl Root {
    fn new(name: String) -> Root {
        Root { wikid_version_major: env!("CARGO_PKG_VERSION_MAJOR").to_owned(),
            wikid_version_minor: env!("CARGO_PKG_VERSION_MINOR").to_owned(),
            name,
            public_url: String::new(),
            bg_image: None,
            fonts: None,
            dark_mode: true,
            font_size: 16,
            main_color: DEFAULT_COLOR.to_owned(),
        }
    }

    pub fn get_root_dir() -> MyResult<String> {
        // Scan for the wikid file.
        let mut path = env::current_dir().unwrap();
        
        loop {
            // Check whether wikid directory exists
            path.push(".wikid");
            if path.exists() {
                path.pop();
                break Ok(path.as_os_str().to_str().expect("Path was corrupted").to_owned())
            }
            path.pop();

            // Advance
            if !path.pop() {
                break Err("Not initialized in wikid directory".to_owned());
            };
        }
    }

    /// Get the URL-formatted link to a file inside the website. If `public` is formatted, it will 
    /// be an HTTPS link. Otherwise it will be file://.
    pub fn get_link_from_local(&self, local_dir: &str, public: bool) -> MyResult<String> {
        let mut no_space_local = local_dir.to_owned();
        no_space_local = no_space_local.replace(b'%' as char, &format!("%{:02x}", b'%'));
        for i in 32..47u8 {
            if i == b'%' {continue;}
            if i == b'#' {continue;}
            no_space_local = no_space_local.replace(i as char, &format!("%{:02x}", i));
        }
        for i in 58..65u8 {
            no_space_local = no_space_local.replace(i as char, &format!("%{:02x}", i));
        }
        for i in 91..97u8 {
            no_space_local = no_space_local.replace(i as char, &format!("%{:02x}", i));
        }
        for i in 123..127u8 {
            no_space_local = no_space_local.replace(i as char, &format!("%{:02x}", i));
        }
        match public {
            true => Ok(format!("{}/{}", self.public_url, no_space_local)),
            false => Ok(format!("file://{}/{}", Self::get_root_dir()?, no_space_local))
        }
    }

    pub fn get_path_from_local(local_dir: &str) -> MyResult<String> {
        Ok(format!("{}/{}", Self::get_root_dir()?, local_dir))
    }

    pub fn summon() -> MyResult<Root> {
        let paths = match fs::read_dir(Root::get_path_from_local(".wikid")?) {
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
                        let root_str = match fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(_) => return Err("Could not open wikid.json".to_owned())
                        };
                        return Ok(match serde_json::from_str(&root_str) {
                            Ok(r) => r,
                            Err(e) => return Err(format!("Wikid.json was corrupted ({})", e))
                        })
                    }
                }
            }
        }
        Err("Could not find the wikid.json file.".to_owned())
    }

    pub fn write(&self) -> MyResult<()> {
        let json_text = match serde_json::to_string_pretty(self) {
            Err(_) => return Err("Failed to write root data to json".to_owned()),
            Ok(t) => t
        };

        let mut file = match File::create(Root::get_path_from_local(".wikid/wikid.json")?) {
            Ok(f) => f,
            Err(_) => return Err("Could not create wikid.json".to_owned())
        };
        if let Err(_) = file.write_all(json_text.as_bytes()) {
            return Err("Could not write to wikid.json".to_owned());
        }

        Ok(())
    }

    #[allow(dead_code)]
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

    /// Returns the name of the current section
    pub fn get_section(&self, path: &str) -> String {
        let path = Path::new(path);
        path.parent().unwrap().file_name().unwrap().to_str().unwrap().to_owned()
    }

    /// Returns the names of all sections
    pub fn get_sections(&self) -> Vec<Section> {
        let mut sections = Vec::new();
        self.get_sections_in_dir(Path::new(&Self::get_root_dir().unwrap()), &mut sections);
        sections
    }

    /// Returns the sections of all sections
    pub fn get_sections_in_dir(&self, path: &Path, sections: &mut Vec<Section>) {
        for file in fs::read_dir(path).unwrap() {
            let file_path = file.unwrap().path();
            if fs::metadata(&file_path).unwrap().is_dir() {
                self.get_sections_in_dir(&file_path, sections);
            } else {
                if file_path.file_name().unwrap() == ".wikid.json" {
                    let text = fs::read_to_string(&file_path).unwrap();
                    let save_section = serde_json::from_str(&text).unwrap();
                    let name = file_path.parent().unwrap().file_name().unwrap().to_str().to_owned().unwrap();
                    sections.push(Section::from_save_section(save_section, name))
                }
            }
        }
    }

    // pub fn rename(&mut self, matches: &ArgMatches) -> MyResult<()> {
    //     let name = matches.value_of("name").expect("name is a required argument").to_string();
    //     self.name = name;
    //     self.write()
    // }
}

impl InitSettings {
    pub fn run(&self) -> MyResult<()> {
        unsafe {
            crate::VERBOSE = self.verbose;
        }
        if let Err(_) = fs::create_dir(".wikid") {
            return Err("Could not create .wikid directory".to_owned());
        }

        if let Err(_) = fs::create_dir("code") {
            return Err("Could not create code directory".to_owned());
        }

        if let Err(_) = fs::create_dir("html") {
            return Err("Could not create target directory".to_owned());
        }

        if let Err(_) = fs::create_dir("text") {
            return Err("Could not create target directory".to_owned());
        }

        if let Err(_) = File::create("text/_toc.md") {
            return Err("Could not create table of contents".to_owned());
        }

        if let Err(_) = fs::write(".gitignore", "text/\n.wikid/\n.gitignore".as_bytes()) {
            return Err("Could not create .gitignore".to_owned())
        }

        if !self.nogit {
            if let Err(_) = Repository::init(".") {
                return Err("Could not initialize github repository in this directory".to_owned());
            }
        }
        else {
            println!("Creating wiki without github integration");
        }
        println!("Created wiki {}", self.name);

        let root = Root::new(self.name.clone());

        root.write()
    }
}
