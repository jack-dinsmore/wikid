use crate::root::Root;

use std::str::FromStr;
use std::io::{BufReader, Lines};
use std::path::PathBuf;
use crate::build::file_queue::FileQueue;
use std::fs;

pub struct CompileTree<'a> {
    arena: Vec<Node<'a>>,
}

struct Node<'a> {
    id: usize,
    parent: Option<usize>,
    children: Vec<usize>,
    name: &'a str,
}

struct TreeIter<'a> {
    tree: CompileTree<'a>,

    path: Vec<usize>,
    names: Vec<&'a str>,
    node: &'a Node<'a>,
}

impl<'a> TreeIter<'a> {
    fn new(tree: CompileTree<'a>) -> Self {
        let path = Vec::new();
        let names = Vec::new();
        let node = &tree.arena[0];
        while !node.children.is_empty() {
            path.push(0);
            names.push(node.name);
            node = &tree.arena[node.children[0]];
        }
        return Self {tree, path, names, node}
    }
}

impl<'a> Iterator for TreeIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let next_node = self.node;

        // Find a layer with one more child
        loop {
            let parent = &self.tree.arena[match next_node.parent {
                Some(p) => p,
                None => return None,
            }];
            let next_index = self.path.pop().expect("Tree iterator path corrupted") + 1;
            if parent.children.len() >= next_index {
                // Go up one level
                next_node = parent;
                self.names.pop();
                continue;
            }
            else {
                // Choose the next child
                next_node = &self.tree.arena[parent.children[next_index]];
                self.path.push(next_index);
                break;
            }
        }

        // Descend to leaf
        while !next_node.children.is_empty() {
            self.names.push(next_node.name);
            self.path.push(0);
            next_node = &self.tree.arena[next_node.children[0]];
        }

        self.node = next_node;

        Some(self.names.join('/'))
    }
}

impl<'a> CompileTree<'a> {
    pub fn new(root: &Root) -> CompileTree {
        let tree = Self::blank('.');

        tree.arena[0].make(&tree, Vec::new());

        tree
    }

    pub fn compile(&self, file_queue: &mut FileQueue) {
        for path in self.iter() {
            file_queue.add(path, compile_file(path));
        }
    }


    fn blank(root_text: &'a str) -> CompileTree {
        CompileTree {
            arena: vec![Node {
                id: 0,
                parent: None,
                children: Vec::new(),
                name: root_text,
            }],
        }
    }

    fn add_node(&self, parent: &Node, name: &'a str) {
        self.arena.push(Node {
            id: self.arena.len(),
            parent: Some(parent.id),
            children: Vec::new(),
            name
        });

        parent.children.push(self.arena.len());
    }

    fn iter(self) -> TreeIter<'a> {
        TreeIter::new(self)
    }
}

impl<'a> Node<'a> {
    fn make(&self, tree: &CompileTree<'a>, previous_path: Vec<&str>) {
        previous_path.push(self.name);
        let path_str = previous_path.join('/');
        for path in fs::read_dir(path_str).expect("Could not open local paths") {
            let path = path.expect("Could not open path").path();
            let md = fs::metadata(path).expect("Could not open metadata for path");

            tree.add_node(self, path.file_name().expect("Could not extract file name"));
            
            if md.is_dir() {
                tree.arena[self.children[self.children.len()]].make(
                    tree,
                    previous_path);
            }
        }
    }
}

fn compile_file<'a>(path: &'a str) -> &'a str{
    // Turn code in the file "path" into some compiled code in the return type.
}