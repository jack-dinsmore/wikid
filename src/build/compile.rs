use crate::root::Root;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use crate::constants::MyResult;
use crate::build::refs::RefMap;
use crate::build::file_queue::FileQueue;


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

#[derive(Copy, Clone, Debug)]
enum ListType {
    Ordered,
    Unordered,
}

/// An enum for the different kinds of commands that have been implemented
#[derive(Copy, Clone, Debug)]
pub enum CommandTypes {
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
    Label,
    Failed,
}

pub struct Command {
    pub c_type: CommandTypes,
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
            3 => self.link_text.push(c),
            _ => panic!("Progress should not get this high")
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

    fn make(&mut self, ref_map: &RefMap, ) -> MyResult<String> {
        // Guaranteed that self.progress is 3
        let (display_text, href) = match self.link_type {
            '[' => (self.display_text.clone(), self.link_text.clone()),
            '{' => {
                // Internal link
                let (internal_name, internal_link) = match ref_map.get_link(&self.link_text) {
                    Some(i) => i,
                    None => return Err("Could not find link".to_owned())
                };
                if self.display_text.is_empty() {
                    (internal_name, internal_link)
                } else {
                    (self.display_text.clone(), internal_link)
                }
            }
            _ => return Err("Internal link parsing error".to_owned())
        };
        
        // Reset the link
        self.progress = 0;
        self.display_text = "".to_owned();
        self.link_text = "".to_owned();
        self.link_type = '.';

        Ok(format!("<a href={}>{}</a>", href, display_text))
    }

    fn make_img(&mut self, ref_map: &RefMap) -> MyResult<String> {
        // Guaranteed that self.progress is 3
        let href = match self.link_type {
            '[' => self.link_text.clone(),
            '{' => ref_map.url_stem.clone() + &self.link_text, // Internal link
            _ => return Err("Internal link parsing error".to_owned())
        };
        
        // Reset the link
        self.progress = 0;
        self.display_text = "".to_owned();
        self.link_text = "".to_owned();
        self.link_type = '.';

        Ok(format!("<a href=\"{}\"><img src=\"{}\" alt=\"{}\" /></a>", href, href, self.display_text))
    }
}

impl Command {
    pub fn new() -> Command {
        Command {c_type: CommandTypes::Evaluating }
    }
    pub fn parse_command(&mut self, c: char) -> bool {
        match self.c_type {
            CommandTypes::Evaluating => (),
            CommandTypes::HeaderProgress(_) => (),
            CommandTypes::CodeProgress(_) => (),
            CommandTypes::OrderedListProgress => (),
            CommandTypes::MultiLatexProgress => (),
            CommandTypes::Failed => return false,
            _ => return false,
        };
        self.c_type = match &self.c_type {
            CommandTypes::Evaluating => match c {
                '#' => CommandTypes::HeaderProgress(1),
                '>' => CommandTypes::BlockQuote,
                '`' => CommandTypes::CodeProgress(1),
                '1' => CommandTypes::OrderedListProgress,
                '*' => CommandTypes::List(ListType::Unordered),
                '!' => CommandTypes::Image,
                '$' => CommandTypes::MultiLatexProgress,
                '%' => CommandTypes::Comment,
                '~' => CommandTypes::Label,
                _ => CommandTypes::NoCommand
            },
            CommandTypes::HeaderProgress(i) => match c {
                '#' => CommandTypes::HeaderProgress(i+1),
                _ => if *i < 5 {CommandTypes::Header(*i)} else {CommandTypes::Failed}
            },
            CommandTypes::CodeProgress(i) => match c {
                '`' => CommandTypes::CodeProgress(i+1),
                _ => if *i == 3 {CommandTypes::Code} else {CommandTypes::Failed}
            },
            CommandTypes::OrderedListProgress => match c {
                '.' => CommandTypes::List(ListType::Ordered),
                _ => CommandTypes::Failed,
            },
            CommandTypes::MultiLatexProgress => match c {
                '$' => CommandTypes::MultiLatex,
                _ => CommandTypes::Failed,
            },
            _ => self.c_type
        };
        true
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
            _ => "".to_owned(),
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
            CommandTypes::List(_) => "</li>".to_owned(),
            CommandTypes::Image => "".to_owned(),
            CommandTypes::MultiLatex => "\\]".to_owned(),
            _ => "".to_owned(),
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
                        _ => panic!("Unknown modifier")
                    }));
                }
                else {
                    return Err("Modifier misformatting".to_owned());
                }
            }
            return Ok(Some(match c {
                '*' => "<b>".to_owned(),
                '_' => "<i>".to_owned(),
                '`' => "<code>".to_owned(),
                '$' => "\\(".to_owned(),
                _ => panic!("Unknown modifier")
            }));
        }
        Ok(None)
    }

    fn is_latex(&self) -> bool {
        self.modifiers.contains(&'$')
    }
}


impl ParseState {
    fn new() -> ParseState {
        ParseState {list: None, previous_paragraph: false}
    }
    fn terminal(&self) -> String{
        if let Some(l) = self.list {
            match l {
                ListType::Ordered => "</ol>".to_owned(),
                ListType::Unordered => "</ul>".to_owned()
            }
        } else {
            "".to_owned()
        }
    }
}


pub fn compile_file<'a>(path: &'a str, ref_map: &RefMap, public: bool) -> MyResult<String> {
    // Turn code in the file "path" into some compiled code in the return type.
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Err(format!("Compile tree was corrupted: path {}", path))
    };

    let file_name = Path::new(path).file_name().expect("Incorrectly formatted path");

    let root = Root::summon()?;
    let section_name = root.get_section(path);
    let css_name = if public {
        format!("{}/css/{}.css", root.public_url.clone(), section_name)
    } else {
        format!("file://{}/html/css/{}.css", Root::get_root_dir().expect("No root directory"), section_name)
    };

    let mut compiled_text : String = format!("<html>
<head>
    <meta charset=\"utf-8\">
    <link rel = \"stylesheet\" type = \"text/css\" href = \"{}\">
    <script src=\"https://polyfill.io/v3/polyfill.min.js?features=es6\"></script>
    <script id=\"MathJax-script\" async
            src=\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js\">
    </script>
    <title>{}</title>
</head><body>", css_name, file_name.to_str().expect("Incorrectly formatted path"));

    let mut parse_state = ParseState::new();

    // Write header material

    // Write center material
    for (line_num, line) in io::BufReader::new(file).lines().enumerate() {
        compiled_text.push_str(&match parse_line(line.expect("Error in reading files"), ref_map, &mut parse_state) {
            Ok(l) => l,
            Err(m) => return Err(format!("File {} line {}: {}", path, line_num+1, m)),
        });
        compiled_text.push_str("\n");
    }
    compiled_text.push_str(&parse_state.terminal());

    // Write footer material
    
    compiled_text += "</body></html>\n";

    Ok(compiled_text)
}

fn parse_line(uncompiled_line: String, ref_map: &RefMap, parse_state: &mut ParseState) -> MyResult<String> {
    let mut escaped = false;
    let mut possible_link = PossibleLink::new();
    let mut result = "".to_owned();
    let mut before = "".to_owned();
    let mut command = Command::new();
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
        else if let CommandTypes::Label = command.c_type {
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
                        CommandTypes::Image => possible_link.make_img(ref_map)?,
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
                    ListType::Unordered => before = format!("</ul><ol>{}", result)
                },
                ListType::Unordered => match old_type {
                    ListType::Unordered => (),
                    ListType::Ordered => before = format!("</ol><ul>{}", before)
                }
            },
            None => {// Start new list
                match list_type {
                    ListType::Ordered => before = format!("<ol>{}", before),
                    ListType::Unordered => before = format!("<ul>{}", before)
                }
            }
        };
        parse_state.list = Some(list_type);
    }
    else {
        match parse_state.list {
            Some(old_type) => match old_type{
                ListType::Ordered => before = format!("</ol>{}", before),
                ListType::Unordered => before = format!("</ul>{}", before)
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
    
    Ok(format!("{}{}{}{}", before, command.prefix(), result, command.postfix()))
}
