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
            let name = key.split("/").collect::<Vec<_>>();
            let name = name[..name.len()-1].join("/");
            if let Err(_) = create_dir(format!("{}{}", target_dir, name)) {
                return Err("Could not create directory");
            };
        }

        // Write files
        for (key, value) in &self.map {
            let mut f = match File::create(format!("{}{}", target_dir, key)) {
                Ok(f) => f,
                Err(_) => return Err("Could not create all files")
            };

            if let Err(_) = f.write_all(value.as_bytes()) {
                return Err("Could not write to all files");
            };
        }
        Ok(())
    }

    pub fn add(&mut self, name: String, text: String) {
        self.map.insert(name, text);
    }


}
