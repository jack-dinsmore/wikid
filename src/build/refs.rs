use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::io::{self, BufRead};
use crate::constants::MyResult;
use crate::build::compile::{Command, CommandTypes};
use crate::root::Root;

#[derive(Debug)]
pub struct RefMap {
    posts: HashMap<String, (String, String)>,// interior_label, (exterior_label, link)
    secs: HashMap<String, (String, [u32;5], String)>,// interior_label, (exterior_label, number, link)
    eqs: HashMap<String, (u32, String)>,// interior_Label, (eqn number, link)
    figures: HashMap<String, (u32, String)>,// interior_Label, (eqn number, link)
    projects: HashMap<String, (String, String)>,// interior_label, (external_label, link)
    vocab: HashMap<String, String>,// display name, link
    public: bool,
}

impl RefMap {
    pub fn new (public: bool) -> RefMap {
        let posts = HashMap::new();
        let secs = HashMap::new();
        let eqs = HashMap::new();
        let projects = HashMap::new();
        let vocab = HashMap::new();
        let figures = HashMap::new();

        RefMap { posts, secs, eqs, projects, vocab, figures, public }
    }

    /// Scan through file looking for sections, equations, projects, and vocab.
    pub fn add_file(&mut self, local_path: &str) -> MyResult<()> {
        let global_path = Root::get_path_from_local(local_path)?;
        let file = match File::open(&global_path) {
            Ok(f) => f,
            Err(_) => return Err(format!("Compile tree was corrupted in refs: path {}", global_path))
        };

        let file_name = Path::new(&global_path).file_name().expect("Incorrectly formatted path");
        let root = Root::summon()?;

        let mut label = "".to_owned();
        let mut eq_num = 1;
        let mut fig_num = 1;
        let mut sec_num = [0; 5];
        let mut header_index = 0;
        let mut bare_link = format!("html/{}", &local_path[5..local_path.len()-3]);// Remove text/ and .md
        if bare_link.ends_with("_toc") {
            bare_link = format!("{}index", &bare_link[..bare_link.len()-4]);
        }

        self.posts.insert((local_path[5..local_path.len()-3]).to_owned(), (
            file_name.to_str().expect("Incorrectly formatted path").to_owned(),
            root.get_link_from_local(&format!("{}.html", bare_link), self.public)?
        ));

        // Write center material
        for (line_num, line) in io::BufReader::new(file).lines().enumerate() {
            let line = line.expect("Error in reading files");

            let mut command = Command::new();
            let mut command_arg = "".to_owned();
            for c in line.chars() {
                if !command.parse_command(c) {
                    if c != ' ' {
                        command_arg.push(c);
                    }
                }
            }

            match &command.c_type {
                CommandTypes::Header(i) => { 
                    if *i < header_index {
                        for j in i+1..5 {
                            sec_num[j as usize] = 0;
                        }
                    }
                    header_index = *i;
                    sec_num[*i as usize] += 1;
                },
                CommandTypes::MultiLatex => eq_num += 1,
                _ => ()
            };

            if !label.is_empty() {
                match &command.c_type {
                    CommandTypes::Header(_) => {self.secs.insert(label.to_owned(), (
                        command_arg,
                        sec_num,
                        root.get_link_from_local(
                            &format!("{}.html-sec-{}", bare_link, sec_num.iter().map( |&n| n.to_string() + "-").collect::<String>()),
                            self.public)?
                    ));},
                    CommandTypes::MultiLatex => {self.eqs.insert(label.to_owned(), (
                        eq_num,
                        root.get_link_from_local(
                            &format!("{}.html#eq{}", bare_link, eq_num), self.public)?
                    )); eq_num += 1;},
                    CommandTypes::Image => {self.figures.insert(label.to_owned(), (
                        fig_num,
                        root.get_link_from_local(
                            &format!("{}.html#fig{}", bare_link, fig_num), self.public)?
                    )); fig_num += 1;},
                    _ => {
                        println!("Unused label {} with command type {:?}", label, command.c_type);
                    }
                }
                label = "".to_owned();
            }

            if let Some(c) = line.chars().nth(0) {
                if c == '~' {
                    // Label has been found
                    label = line[1..].to_owned();
                    loop {
                        match label.chars().nth(0) {
                            Some(c) => if c != ' ' { break } else { label = label[1..].to_owned() },
                            None => return Err(format!("File {} line {}: Label line was empty", local_path, line_num+1))
                        }
                    }
                    loop {
                        match label.chars().last() {
                            Some(c) => if c != ' ' { break } else { label = label[..label.len()-2].to_owned() },
                            None => return Err("Label line was empty".to_owned())
                        }
                    }
                }
                continue;
            }
        }
        Ok(())
    }

    pub fn add_glossary(&mut self, path: &str) -> MyResult<()> {
        let _file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Ok(())// No glossary
        };

        // Load glossary
        unimplemented!();
    }

    pub fn get_link(&self, label: &str) -> Option<(String, String)> {
        if self.posts.contains_key(label) {
            Some((self.posts[label].0.clone(), self.posts[label].1.clone()))
        } else if self.secs.contains_key(label) {
            Some((self.secs[label].0.clone(), self.secs[label].2.clone()))
        } else if self.eqs.contains_key(label) {
            Some((format!("Eq. {}", self.eqs[label].0), self.eqs[label].1.clone()))
        } else if self.projects.contains_key(label) {
            Some((self.projects[label].0.clone(), self.projects[label].1.clone()))
        } else if self.vocab.contains_key(label) {
            Some((label.to_owned(), self.vocab[label].clone()))
        } else if self.figures.contains_key(label) {
            Some((format!("Fig. {}", self.figures[label].0), self.figures[label].1.clone()))
        } else {
            None
        }
    }
}
