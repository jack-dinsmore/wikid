use std::fs::{create_dir, File};
use std::collections::HashMap;
use crate::constants::MyResult;
use crate::root::Root;
use std::io::Write;

pub struct FileQueue {
    map: HashMap<String, String>,
}

impl FileQueue {
    pub fn new() -> FileQueue {
        FileQueue { map: HashMap::new() }
    }

    pub fn write(self, target_dir: &str) -> MyResult<()> {
        // Make directories
        for key in self.map.keys() {
            let dir_name = format!("{}{}", target_dir, key);
            let mut name = dir_name.split("/").collect::<Vec<_>>();
            name.retain(|&x| x != ".");
            for i in 1..name.len() {
                let joined_name = name[..i].join("/");
                if let Err(_) = create_dir(&joined_name) {
                    // Directory probably already existed
                };
            }
        }

        // Write files
        for (key, value) in &self.map {
            let mut f = match File::create(format!("{}{}", target_dir, key)) {
                Ok(f) => f,
                Err(_) => return Err("Could not create all files".to_owned())
            };

            if let Err(_) = f.write_all(value.as_bytes()) {
                return Err("Could not write to all files".to_owned());
            };
        }
        Ok(())
    }

    pub fn add(&mut self, name: String, text: String) {
        self.map.insert(name, text);
    }

    pub fn size(&self) -> usize {
        self.map.len()
    }
}
