use crate::root::Root;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use crate::constants::MyResult;
use crate::build::refs::RefMap;
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

     // Helper variables
    path: Vec<usize>,
    names: Vec<&'a str>,
    node: &'a Node<'a>,
}

/// A struct for parsing links
struct PossibleLink {
    display_text: String,
    link_text: String,
    link_type: char, // either [ or {
    progress: u8 // zero for no link, 1 for first part, 2 for intermediate, 3 for second part.
}

enum LinkReturn {
    Pushed,
    Failed(String),
    Done,
    Pass
}

enum ListType {
    Ordered,
    Unordered,
}

/// An enum for the different kinds of commands that have been implemented
enum CommandTypes {
    // https://www.markdownguide.org/basic-syntax/
    NoCommand,
    Evaluating,
    Header(u8),
    HeaderProgress(u8),
    BlockQuote,
    List(ListType),
    OrderedListProgress,
    Code,
    CodeProgress(u8),
    Image,
    MultiLatex,
    MultiLatexProgress,
    Comment,
    Failed,
}

struct Command {
    c_type: CommandTypes,
    letters: String,
}

struct Modifiers {
    modifiers: Vec<char>,
}

struct ParseState {
    list: Option<ListType>,
    previous_paragraph: bool,
}

impl PossibleLink {
    fn new() -> PossibleLink {
        PossibleLink{ display_text: "".to_owned(), link_text: "".to_owned(),
            link_type: '.', progress: 0 }
    }

    /// Tries to add a character. If fails, returns the text to be written. Otherwise returns None.
    fn try_add(&mut self, c: char) -> LinkReturn {
        match c {
            '[' => {
                if self.progress == 0 {
                    self.display_text = "".to_owned();
                    self.progress = 1;
                    return LinkReturn::Pushed;
                }
                else {
                    return LinkReturn::Failed(self.clear());
                }
            },

            '(' => {
                if self.progress == 2 {
                    self.link_type = c;
                    self.progress = 3;
                    return LinkReturn::Pushed;
                }
                else {
                    return LinkReturn::Failed(self.clear());
                }
            },

            '{' => {
                if self.progress == 2 {
                    self.link_type = c;
                    self.progress = 3;
                    return LinkReturn::Pushed;
                }
                else {
                    return LinkReturn::Failed(self.clear());
                }
            },


            ']' => {
                if self.progress == 1 {
                    self.progress = 2;
                    return LinkReturn::Pushed;
                }
                else {
                    return LinkReturn::Failed(self.clear());
                }
            },

            ')' => {
                if self.progress == 3 && self.link_type == '(' {

                    return LinkReturn::Done;
                }
                else {
                    return LinkReturn::Failed(self.clear());
                }
            },

            '}' => {
                if self.progress == 3 && self.link_type == '{' {
                    self.progress = 2;
                    return LinkReturn::Done;
                }
                else {
                    return LinkReturn::Failed(self.clear());
                }
            },

            _ => (),
        }
        match self.progress {
            0 => return LinkReturn::Pass,
            1 => self.display_text.push(c),
            2 => return LinkReturn::Failed(self.clear()),
            3 => self.link_text.push(c)
        };
        LinkReturn::Pushed
    }

    fn clear(&mut self) -> String {
        let mut out = "(".to_owned();
        out.push_str(&self.display_text);
        self.display_text = "".to_owned();
        if self.progress == 2 {
            out.push(')');
        }
        if self.progress == 3 {
            out.push(self.link_type);
            out.push_str(&self.link_text);
        }
        self.progress = 0;
        self.display_text = "".to_owned();
        self.link_text = "".to_owned();
        self.link_type = '.';
        out
    }

    fn make(&mut self, ref_map: &RefMap) -> MyResult<String> {
        // Guaranteed that self.progress is 3
        let href = match self.link_type {
            '[' => self.link_text,
            '{' => {
                // Internal link
                /// Logic
            }
            _ => return Err("Internal link parsing error")
        };
        
        // Reset the link
        self.progress = 0;
        self.display_text = "".to_owned();
        self.link_text = "".to_owned();
        self.link_type = '.';

        format!("<a href={}>{}</a>", href, self.display_text)
    }

    fn make_img(&mut self) -> MyResult<String> {
        // Guaranteed that self.progress is 3
        let href = match self.link_type {
            '[' => self.link_text,
            '{' => {
                // Internal link
                /// Logic
            }
            _ => return Err("Internal link parsing error")
        };
        
        // Reset the link
        self.progress = 0;
        self.display_text = "".to_owned();
        self.link_text = "".to_owned();
        self.link_type = '.';

        format!("<a href=\"{}\"><img src=\"{}\" alt=\"{}\" /></a>", href, href, self.display_text)
    }
}

impl Command {
    fn new() -> Command {
        Command {c_type: CommandTypes::Evaluating, letters: "".to_owned() }
    }
    fn parse_command(&mut self, c: char) -> bool {
        self.c_type = match self.c_type {
            CommandTypes::Evaluating => match c {
                '#' => CommandTypes::HeaderProgress(1),
                '>' => CommandTypes::BlockQuote,
                '`' => CommandTypes::CodeProgress(1),
                '1' => CommandTypes::OrderedListProgress,
                '*' => CommandTypes::List(ListType::Unordered),
                '!' => CommandTypes::Image,
                '$' => CommandTypes::MultiLatexProgress,
                '%' => CommandTypes::Comment,
                _ => CommandTypes::NoCommand
            },
            CommandTypes::HeaderProgress(i) => match c {
                '#' => CommandTypes::HeaderProgress(i+1),
                _ => if i < 5 {CommandTypes::Header(i)} else {CommandTypes::Failed}
            },
            CommandTypes::CodeProgress(i) => match c {
                '`' => CommandTypes::CodeProgress(i+1),
                _ => if i == 3 {CommandTypes::Code} else {CommandTypes::Failed}
            },
            CommandTypes::OrderedListProgress => match c {
                '.' => CommandTypes::List(ListType::Ordered),
                _ => CommandTypes::Failed,
            },
            CommandTypes::MultiLatexProgress => match c {
                '$' => CommandTypes::MultiLatex,
                _ => CommandTypes::Failed,
            },
        };
        match self.c_type {
            CommandTypes::NoCommand => false,
            CommandTypes::Failed => false,
            _ => {self.letters.push(c); true}
        }
    }

    fn prefix(&self) -> String {
        match self.c_type {
            CommandTypes::Header(i) => match i {
                1 => "<h1>".to_owned(),
                2 => "<h2>".to_owned(),
                3 => "<h3>".to_owned(),
                4 => "<h4>".to_owned(),
                5 => "<h5>".to_owned(),
                _ => panic!("Header was out of range")
            },
            CommandTypes::BlockQuote => "<blockquote>".to_owned(),
            CommandTypes::Code => "<code>".to_owned(),
            CommandTypes::List(_) => "<li>".to_owned(),
            CommandTypes::Image => "".to_owned(),
            CommandTypes::MultiLatex => "\\[".to_owned(),
            _ => self.letters,
        }
    }

    fn postfix(&self) -> String {
        match self.c_type {
            CommandTypes::Header(i) => match i {
                1 => "</h1>".to_owned(),
                2 => "</h2>".to_owned(),
                3 => "</h3>".to_owned(),
                4 => "</h4>".to_owned(),
                5 => "</h5>".to_owned(),
                _ => panic!("Header was out of range")
            },
            CommandTypes::BlockQuote => "</blockquote>".to_owned(),
            CommandTypes::Code => "</code>".to_owned(),
            CommandTypes::List(_) => "<li>".to_owned(),
            CommandTypes::Image => "".to_owned(),
            CommandTypes::MultiLatex => "\\]".to_owned(),
            _ => self.letters,
        }
    }
}

impl Modifiers {
    fn new() -> Modifiers{
        Modifiers {modifiers: Vec::new()}
    }

    fn check(&mut self, c: char) -> MyResult<Option<String>> {
        if c == '*' || c == '_' || c == '`' || c == '$' {
            if self.modifiers.contains(&c) {
                if *self.modifiers.last().unwrap() == c {
                    self.modifiers.pop();
                    return Ok(Some(match c {
                        '*' => "</b>".to_owned(),
                        '_' => "</i>".to_owned(),
                        '`' => "</code>".to_owned(),
                        '$' => "\\)".to_owned(),
                    }));
                }
                else {
                    return Err("Modifier misformatting");
                }
            }
            return Ok(Some(match c {
                '*' => "<b>".to_owned(),
                '_' => "<i>".to_owned(),
                '`' => "<code>".to_owned(),
                '$' => "\\(".to_owned(),
            }));
        }
        Ok(None)
    }

    fn is_latex(&self) -> bool {
        self.modifiers.contains(&'$')
    }
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
            let parent = self.tree.arena[match next_node.parent {
                Some(p) => p,
                None => return None,
            }];
            let next_index = self.path.pop().expect("Tree iterator path corrupted") + 1;
            if parent.children.len() >= next_index {
                // Go up one level
                next_node = &parent;
                self.names.pop();
                continue;
            }
            else {
                // Choose the next child
                let child = self.tree.arena[parent.children[next_index]];
                next_node = &child;
                self.path.push(next_index);
                break;
            }
        }

        // Descend to leaf
        while !next_node.children.is_empty() {
            self.names.push(next_node.name);
            self.path.push(0);
            let first_child = self.tree.arena[next_node.children[0]];
            next_node = &first_child;
        }

        self.node = next_node;

        Some(&self.names.join("/"))
    }
}

impl<'a> CompileTree<'a> {
    pub fn new(root: &'a Root) -> CompileTree {
        let tree = Self::blank(".");

        tree.arena[0].make(&tree, Vec::new());

        tree
    }

    pub fn compile(&self, file_queue: &mut FileQueue, ref_map: &RefMap) -> MyResult<()> {
        for path in self.iter() {
            file_queue.add(path.to_owned(), compile_file(path, ref_map)?);
        }
        Ok(())
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
    fn make(&self, tree: &CompileTree<'a>, previous_path: Vec<&'a str>) {
        previous_path.push(self.name);
        let path_str = previous_path.join("/");
        for path in fs::read_dir(path_str).expect("Could not open local paths") {
            let path = path.expect("Could not open path").path();
            let md = fs::metadata(path).expect("Could not open metadata for path");

            tree.add_node(self, match path.file_name().expect("Could not extract file name").to_str() {
                Some(s) => s,
                None => panic!("Could not convert file name into string")
            });
            
            if md.is_dir() {
                tree.arena[self.children[self.children.len()]].make(
                    tree,
                    previous_path);
            }
        }
    }
}

impl ParseState {
    fn new() -> ParseState {
        ParseState {list: None, previous_paragraph: false}
    }
}


fn compile_file<'a>(path: &'a str, ref_map: &RefMap) -> MyResult<String> {
    // Turn code in the file "path" into some compiled code in the return type.
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Err("Compile tree was corrupted")
    };
    let mut compiled_text : String = "".to_owned();
    let mut parse_state = ParseState::new();

    // Write header material
    unimplemented!();

    // Write center material
    for (line_num, line) in io::BufReader::new(file).lines().enumerate() {
        compiled_text.push_str(match parse_line(line.expect("Error in reading files"), ref_map, &parse_state) {
            Ok(l) => &l,
            Err(m) => return Err(format!("Line {}: {}", line_num, m)),
        });
        compiled_text.push_str("\n");
    }

    // Write footer material
    unimplemented!();

    Ok(compiled_text)
}

fn parse_line(uncompiled_line: String, ref_map: &RefMap, parse_state: &ParseState) -> MyResult<String> {
    let mut escaped = false;
    let mut possible_link = PossibleLink::new();
    let result = "".to_owned();
    let command = Command::new();
    let mut modifiers = Modifiers::new();


    if uncompiled_line.is_empty(){
        if parse_state.previous_paragraph {
            parse_state.previous_paragraph = false;
            return Ok("</p>".to_owned());
        }
    }

    for c in uncompiled_line.chars() {
        if command.parse_command(c) {
            continue;
        }
        else if let CommandTypes::Comment = command.c_type {
            return Ok("".to_owned());
        }
        else if modifiers.is_latex() {
            result.push(c);
        }
        else if let CommandTypes::MultiLatex = command.c_type {
            result.push(c);
        }
        else if escaped {
            result.push(c);
            escaped = false;
        }
        else {
            match c {
                '\\' => escaped = true,
                _ => match possible_link.try_add(c) { // Try a link
                    LinkReturn::Pushed => (),
                    LinkReturn::Failed(s) => result.push_str(&s),
                    LinkReturn::Done => result.push_str(&match command.c_type {
                        CommandTypes::Image => possible_link.make_img()?,
                        _ => possible_link.make(ref_map)?
                    }),
                    LinkReturn::Pass => match modifiers.check(c)? {
                        Some(s) => result.push_str(&s),
                        None => result.push(c) 
                    }// Check if it's a bold modifier
                }
            };
        }
    }
    if let CommandTypes::List(list_type) = command.c_type {
        match parse_state.list {
            Some(old_type) => match list_type {
                ListType::Ordered => match old_type {
                    ListType::Ordered => (),
                    ListType::Unordered => result = format!("</ul><ol>{}", result)
                },
                ListType::Ordered => match old_type {
                    ListType::Ordered => (),
                    ListType::Unordered => result = format!("</ol><ul>{}", result)
                }
            },
            None => {// Start new list
                match list_type {
                    ListType::Ordered => result = format!("<ol>{}", result),
                    ListType::Unordered => result = format!("<ul>{}", result)
                }
            }
        };
        parse_state.list = Some(list_type);
    }
    else {
        match parse_state.list {
            Some(old_type) => match old_type{
                ListType::Ordered => result = format!("</ol>{}", result),
                ListType::Unordered => result = format!("</ul>{}", result)
            },
            None => ()
        };
        parse_state.list = None;
    }
    if let CommandTypes::NoCommand = command.c_type {
        if !parse_state.previous_paragraph {
            result = format!("<p>{}", result);
            parse_state.previous_paragraph = true;
        }
    }
    if let CommandTypes::Failed = command.c_type {
        if !parse_state.previous_paragraph {
            result = format!("<p>{}", result);
            parse_state.previous_paragraph = true;
        }
    }
    
    Ok(format!("{}{}{}", command.prefix(), result, command.postfix()))
}
