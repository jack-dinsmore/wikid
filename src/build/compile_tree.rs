use crate::root::Root;

use crate::constants::MyResult;
use crate::build::refs::RefMap;
use crate::build::file_queue::FileQueue;
use crate::build::compile::compile_file;
use std::fs;

#[derive(PartialEq, Debug)]
pub struct Node {
    children: Vec<Box<Node>>,
    name: String,
    is_leaf: bool,
}

struct TreeIter<'a> {
    new: bool,
     // Helper variables
    path: Vec<&'a Node>,
    node: &'a Node,
}

impl<'a> TreeIter<'a> {
    fn new(root_node: &'a Node) -> Self {
        let mut path = Vec::new();
        let mut node : &'a Node = root_node;
        while !node.children.is_empty() {
            path.push(node);
            node = &node.children[0];
        }
        path.push(node);
        return Self {new: true, path, node}
    }
}

impl<'a> Iterator for TreeIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.new {
            let mut next_node = self.node;

            // Find a layer with one more child
            loop {
                if self.path.len() < 2 {
                    return None;
                }
                let parent = self.path[self.path.len() - 2];
                self.path.pop().expect("Tree iterator path corrupted");

                let mut take_next_child = false;
                let mut took_child = false;
                for child in &parent.children {
                    if take_next_child {
                        self.path.push(&child);
                        next_node = &child;
                        took_child = true;
                        break;
                    }
                    if **child == *next_node {
                        // Take the next child
                        take_next_child = true;
                    }
                }
                if took_child {
                    break;
                }
            }

            // Descend to leaf
            while !next_node.children.is_empty() {
                self.path.push(&next_node.children[0]);
                next_node = &next_node.children[0];
            }

            self.node = next_node;
        }
        else {
            self.new = false;
        }

        let mut text : String = "text/".to_owned();
        let mut previous = false;
        for n in &self.path {
            if previous {
                text += "/";
            }
            previous = true;
            text = text + &n.name[..];
        }
        if !self.path.last().expect("Path was corrupted").is_leaf {
            return self.next();
        }
        Some(text)
    }
}

impl Node {
    pub fn new() -> Node {
        let mut tree = Node { children: Vec::new(), name: ".".to_owned(), is_leaf: false};

        tree.make(Root::concat_root_dir("text").expect("Could not find root dir"));

        tree
    }

    fn make (&mut self, node_path: String) {
        for new_path in fs::read_dir(node_path).expect("Could not open local paths") {
            let new_path = new_path.expect("Could not open path").path();
            
            if match new_path.as_path().extension() {
                Some(e) => e == "md",
                None => false
            }{
                self.children.push(Node::new_node(match new_path.as_path().file_name() {
                    Some(f) => f.to_str().expect("String did not convert").to_owned(),
                    None => panic!("Could not find a file name")
                }, true));
            }
            
            let md = fs::metadata(&new_path).expect("Could not open metadata for path");
            if md.is_dir() {
                let mut new_node = Node::new_node(match new_path.as_path().file_name() {
                    Some(f) => f.to_str().expect("String did not convert").to_owned(),
                    None => panic!("Could not find a file name")
                }, false);
                new_node.make(new_path.into_os_string().into_string().expect("String did not convert"));
                self.children.push(new_node)
            }
        }
    }

    pub fn compile(&self, file_queue: &mut FileQueue, ref_map: &RefMap) -> MyResult<()> {
        for path in self.iter() {
            file_queue.add(path.to_owned(), compile_file(&path, ref_map)?);
        }
        Ok(())
    }

    pub fn ref_map(&self, root: &Root, public: bool) -> MyResult<RefMap> {
        let mut ref_map = RefMap::new(root, public);
        for path in self.iter() {
            ref_map.add_file(&path)?;
        }
        ref_map.add_glossary(&Root::concat_root_dir("_glossary.md").expect("Root directory corrupted"))?;
        Ok(ref_map)
    }

    fn iter<'a>(&'a self) -> TreeIter<'a> {
        TreeIter::new(self)
    }

    fn new_node(name: String, is_leaf: bool) -> Box<Node> {
        Box::new(Node { children: Vec::new(), name, is_leaf})
    }

    pub fn size(&self) -> usize {
        let mut i = 0;
        for _ in self.iter() {
            i += 1;
        }
        i
    }
}