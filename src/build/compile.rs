use crate::root::Root;

use std::fs::File;
use crate::build::file_queue::FileQueue;
use std::io::{self, BufRead};
use std::path::Path;
use crate::constants::MyResult;
use crate::build::refs::RefMap;
use chrono::Datelike;


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
pub enum ListType {
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
    section_open: bool,
    local_path: String,
    imgs: Vec<(String, String)>,
    fig_num: u32,
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
                    None => return Err(format!("Could not find link {}", self.link_text))
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

    fn make_img(&mut self, parse_state: &mut ParseState, ref_map: &RefMap) -> MyResult<String> {
        let sections = self.link_text.split('?').collect::<Vec<&str>>();
        let (image_path, image_width) = if sections.len() == 1 {
            (sections[0], 100)
        } else if sections.len() == 2 {
            (sections[0], match sections[1].parse::<i32>() {
                Ok(i) => i,
                Err(_) => return Err(format!("Argument {} was not an integer", sections[1]))
            })
        } else {
            return Err(format!("Image had too many question marks in it. {}", self.link_text));
        };
        // Guaranteed that self.progress is 3
        let href = match self.link_type {
            '[' => image_path.to_owned(),
            '{' => {parse_state.move_img(image_path)?;
                format!("{}/{}/{}", ref_map.url_stem, parse_state.local_path, image_path)
                }, // Internal link
            _ => return Err("Internal link parsing error".to_owned())
        };
        
        // Reset the link
        self.progress = 0;
        self.link_text = "".to_owned();
        self.link_type = '.';
        
        let res = Ok(format!("<center><a href=\"{href}\"><img width={width}% src=\"{href}\" alt=\"{display}\" /></a>
        <div class=\"caption\" id=\"fig{fig_num}\"><b>Figure {fig_num}:</b> {display}</div></center>",
        href=href, width=image_width, fig_num=parse_state.figure(), display=self.display_text));
        
        self.display_text = "".to_owned();
        res
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
        if let CommandTypes::NoCommand = self.c_type {
            false
        } else {
            true
        }
    }

    fn prefix(&self) -> String {
        match self.c_type {
            CommandTypes::Header(i) => match i {
                1 => "<h1>".to_owned(),
                2 => "<button class=\"collapsible\"><h2>".to_owned(),
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
                2 => "</h2></button><div class=\"section\">".to_owned(),
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
            self.modifiers.push(c);
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
    fn new(path: &str) -> ParseState {
        let mut local_path = Path::new(path).parent().expect("Path had no parent").to_str().expect("Could not extract path").to_owned();
        if local_path.starts_with("./") {
            local_path = (&local_path[2..]).to_owned();
        }
        ParseState {list: None, previous_paragraph: false, section_open: false, local_path, imgs: Vec::new(), fig_num: 0 }
    }
    fn terminal(self, file_queue: &mut FileQueue) -> String{
        let mut out = if let Some(l) = self.list {
            match l {
                ListType::Ordered => "</ol>".to_owned(),
                ListType::Unordered => "</ul>".to_owned()
            }
        } else {
            "".to_owned()
        };
        if self.section_open {
            out = format!("{}</div>", out);
        }

        file_queue.append_imgs(self.imgs);

        out
    }

    fn move_img(&mut self, link_text: &str) -> MyResult<()> {
        let path_from = format!("{}/{}/{}", Root::get_root_dir()?, self.local_path, link_text);
        let path_to = format!("{}/html/{}/{}", Root::get_root_dir()?, self.local_path, link_text);
        if !Path::new(&path_from).exists() {
            return Err(format!("Could not find image {}", path_from))
        }
        self.imgs.push((path_from, path_to));
        Ok(())
    }

    fn figure(&mut self) -> u32 {
        self.fig_num += 1;
        self.fig_num
    }
}

fn get_footer() -> MyResult<String> {
    let root = Root::summon()?;
    Ok(format!("Copyright &copy; {year} Jack Dinsmore. Version {maj}.{min}",
    year=chrono::Utc::now().year(), maj=root.wikid_version_major, min=root.wikid_version_minor))
}

pub fn compile_file<'a>(path: &'a str, file_queue: &mut FileQueue, ref_map: &RefMap, public: bool) -> MyResult<String> {
    // Turn code in the file "path" into some compiled code in the return type.
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Err(format!("Compile tree was corrupted: path {}", path))
    };

    let root = Root::summon()?;
    let section_name = root.get_section(path);

    let root_toc_path = if public {
        format!("{}/index.html", root.public_url.clone())
    } else {
        format!("file://{}/html/index.html", Root::get_root_dir().expect("No root directory"))
    };

    let header = match root.get_section_path(path) {
        Some(p) => {
            let sec_toc_path = if public {
                format!("{}/{}/index.html", root.public_url.clone(), &p[2..])
            } else {
                format!("file://{}/html/{}/index.html", Root::get_root_dir().expect("No root directory"), &p[2..])
            };
            format!("<h2><a href=\"{}\">Home</a> > <a href=\"{}\">{}</a></h2>", root_toc_path, sec_toc_path, section_name)
        },
        None => format!("<h2><a href=\"{}\">Home</a></h2>", root_toc_path)
    };
    let css_name = if public {
        format!("{}/css/{}.css", root.public_url.clone(), section_name)
    } else {
        format!("file://{}/html/css/{}.css", Root::get_root_dir().expect("No root directory"), section_name)
    };

    let mut compiled_text : String = format!(r#"<html>
<head>
    <meta charset="utf-8">
    <link rel="stylesheet" type = "text/css" href = "{css_name}">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=DM+Sans&family=Nunito&display=swap" rel="stylesheet">
    <script src="https://polyfill.io/v3/polyfill.min.js?features=es6"></script>
    <script id="MathJax-script" async
            src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js">
    </script>
    <script>
window.MathJax = {{
    tex: {{
        macros: {{
            bm: ["{{\boldsymbol #1}}",1],
            parens: ["\left( #1 \right)", 1],
            braces: ["\left\{{ #1 \right\}}", 1],
            brackets: ["\left[ #1 \right]", 1],
            eval: ["\left. #1 \right|", 1],
            fraci: ["{{#1}} / {{#2}}", 2],
            expp: ["\exp\left( #1 \right)", 1],
        }}
    }}
}}
    </script>
    <title>{all_name}</title>
</head><body><div id="content">{header}"#,
    css_name=css_name, all_name=root.name, header=header);

    let mut parse_state = ParseState::new(path);

    // Write header material

    // Write center material
    for (line_num, line) in io::BufReader::new(file).lines().enumerate() {
        compiled_text.push_str(&match parse_line(line.expect("Error in reading files"), ref_map, &mut parse_state) {
            Ok(l) => l,
            Err(m) => return Err(format!("File {} line {}: {}", path, line_num+1, m)),
        });
        compiled_text.push_str("\n");
    }
    compiled_text.push_str(&parse_state.terminal(file_queue));

    // Write footer material
    
    compiled_text += "</div><div id=\"footer\">\n";
    compiled_text += &get_footer()?;
    compiled_text += "</div></body>
<script>
var coll = document.getElementsByClassName(\"collapsible\");
var i;

for (i = 0; i < coll.length; i++) {{
    coll[i].addEventListener(\"click\", function() {{
        this.classList.toggle(\"active\");
        var content = this.nextElementSibling;
        if (content.style.maxHeight === \"0px\"){{
            content.style.maxHeight = content.scrollHeight+\"px\";
        }} else {{
            content.style.maxHeight = \"0px\";
        }}
    }});
}}
</script></html>\n";
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
        else if c != '$' && modifiers.is_latex() {
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
                        CommandTypes::Image => possible_link.make_img(parse_state, ref_map)?,
                        _ => possible_link.make(ref_map)?
                    }),
                    LinkReturn::Pass =>  match modifiers.check(c)? {
                        Some(s) => result.push_str(&s),
                        None => result.push(c)
                    }// Check if it's a bold modifier
                }
            };
        }
    }
    if let CommandTypes::Header(i) = command.c_type {
        if i == 1 || i == 2 {
            if parse_state.section_open {
                before = format!("</div> {}", before);
            }
            if i == 2 {
                parse_state.section_open = true;
            }
        }
    }
    if let CommandTypes::List(list_type) = command.c_type {
        match parse_state.list {
            Some(old_type) => match list_type {
                ListType::Ordered => match old_type {
                    ListType::Ordered => (),
                    ListType::Unordered => before = format!("</ul><ol>{}", before)
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
