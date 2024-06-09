use std::{io::{self, Write}, path::Path};

use crate::root::Root;

use super::{compile::ParseState, refs::RefMap, MyResult};

/// A struct for parsing links
#[derive(Debug)]
pub struct PossibleLink {
    pub display_text: String,
    link_text: String,
    link_type: char, // either [ or {
    modifiers: Modifiers,
    progress: u8 // zero for no link, 1 for first part, 2 for intermediate, 3 for second part.
}

/// An enum for keeping track of the state of a link
#[derive(Clone)]
pub enum LinkReturn {
    /// The character was successfully added
    Pushed,
    Child,
    /// This character makes the link invalid
    Failed(String),
    /// The link was successfully concluded and created a footnote
    Footnote(String),
    /// The link was successfully concluded
    Done,
    /// This was not a link
    Pass
}

#[derive(Debug)]
pub struct Modifiers {
    modifiers: Vec<char>,
}

impl Modifiers {
    pub fn new() -> Modifiers{
        Modifiers {modifiers: Vec::new()}
    }

    pub fn check(&mut self, c: char) -> MyResult<Option<String>> {
        if c == '*' || c == '_' || c == '`' || c == '$' || c == ']' {
            if self.modifiers.contains(&c) {
                if *self.modifiers.last().unwrap() == c {
                    self.modifiers.pop();
                    return Ok(Some(match c {
                        '*' => "</b>".to_owned(),
                        '_' => "</i>".to_owned(),
                        '`' => "</code>".to_owned(),
                        '$' => "\\)".to_owned(),
                        ']' => "</div>".to_owned(),
                        _ => return Err(format!("Unknown modifier {}",c))
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
                '[' => "<div>".to_owned(),
                _ => return Err(format!("Unknown modifier {}",c))
            }));
        }
        Ok(None)
    }

    pub fn is_latex(&self) -> bool {
        self.modifiers.contains(&'$')
    }
}

impl PossibleLink {
    pub fn new() -> PossibleLink {
        PossibleLink {
            display_text: "".to_owned(),
            link_text: "".to_owned(),
            link_type: '.',
            modifiers: Modifiers::new(),
            progress: 0,
        }
    }

    /// Tries to add a character. If fails, returns the text to be written. Otherwise returns None.
    pub fn try_add(&mut self, c: char) -> LinkReturn {
        match c {
            '[' => {
                if self.progress == 0 {
                    // Begin the link
                    self.display_text = "".to_owned();
                    self.progress = 1;
                    return LinkReturn::Pushed;
                }
                else {
                    // Brackets are always the first item in a link
                    return LinkReturn::Child;
                }
            },

            '(' => {
                if self.progress == 2 {
                    self.link_type = c;
                    self.progress = 3;
                    return LinkReturn::Pushed;
                }
            },

            '{' => {
                if self.progress == 2 {
                    self.link_type = c;
                    self.progress = 3;
                    return LinkReturn::Pushed;
                }
            },


            ']' => {
                if self.progress == 1 {
                    // End the bracket
                    self.progress = 2;
                    return LinkReturn::Pushed;
                }
            },

            ')' => {
                if self.progress == 3 && self.link_type == '(' {
                    return LinkReturn::Done;
                }
            },

            '}' => {
                if self.progress == 3 && self.link_type == '{' {
                    self.progress = 2;
                    return LinkReturn::Done;
                }
            },

            _ => (),
        }
        match self.progress {
            0 => return LinkReturn::Pass,
            1 => {
                // Perform modifier check
                match self.modifiers.check(c).unwrap() {
                    Some(s) => self.display_text.push_str(&s),
                    None => self.display_text.push(c)
                }
            },
            2 => return self.prep_for_footnote(c),
            3 => self.link_text.push(c),
            _ => panic!("Progress should not get this high")
        };
        LinkReturn::Pushed
    }

    pub fn prep_for_footnote(&mut self, c: char) -> LinkReturn {
        let s = self.clear(c);
        if s.len() >= 2 {
            LinkReturn::Footnote((&s[1..(s.len()-1)]).to_owned())
        } else {
            LinkReturn::Failed(s)
        }
    }

    /// Reset the link and return the current string
    fn clear(&mut self, c: char) -> String {
        let mut out = match self.display_text.is_empty() {
            true => String::new(),
            false => format!("[{}", self.display_text),
        };
        if self.progress == 3 {
            out.push(self.link_type);
            out.push_str(&self.link_text);
        }
        out.push(c);
        self.progress = 0;
        self.display_text = "".to_owned();
        self.display_text = "".to_owned();
        self.link_text = "".to_owned();
        self.link_type = '.';
        out
    }

    pub fn make(&mut self, ref_map: &RefMap, local_path: Option<&str>) -> MyResult<String> {
        // Guaranteed that self.progress is 3
        let (display_text, href) = match self.link_type {
            '(' => (self.display_text.clone(), self.link_text.clone()),
            '{' => {
                // Internal link
                let (internal_name, internal_link) = match ref_map.get_link(&self.link_text, local_path) {
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

    pub fn make_img(&mut self, parse_state: &mut ParseState, public: bool) -> MyResult<String> {
        let root = Root::summon()?;
        let link_parts = self.link_text.split('?').collect::<Vec<&str>>();
        let (image_path, image_width) = if link_parts.len() == 1 {
            (link_parts[0], 100)
        } else if link_parts.len() == 2 {
            (link_parts[0], match link_parts[1].parse::<i32>() {
                Ok(i) => i,
                Err(_) => return Err(format!("Argument {} was not an integer", link_parts[1]))
            })
        } else {
            return Err(format!("Image had too many question marks in it. {}", self.link_text));
        };
        // Guaranteed that self.progress is 3
        let href = match self.link_type {
            '[' => image_path.to_owned(),
            '{' => {parse_state.move_img(image_path)?;
                root.get_link_from_local(&format!("html/{}/{}", &parse_state.local_path[5..], image_path), public)?
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
    
    pub fn make_applet(&mut self, public: bool) -> MyResult<String> {
        let root = Root::summon()?;
        let link_parts = self.link_text.split('?').collect::<Vec<&str>>();
        let (applet_path, applet_width, applet_height) = if link_parts.len() == 1 {
            (link_parts[0], 640, 480)
        } else if link_parts.len() == 3 {
            (link_parts[0], match link_parts[1].parse::<i32>() {
                Ok(i) => i,
                Err(_) => return Err(format!("Argument {} was not an integer", link_parts[1]))
            }, match link_parts[2].parse::<i32>() {
                Ok(i) => i,
                Err(_) => return Err(format!("Argument {} was not an integer", link_parts[2]))
            })
        } else {
            return Err(format!("Applet had too many question marks in it. {}", self.link_text));
        };

        // Process the link
        let rust_path = match self.link_type {
            '[' => applet_path.to_owned(),
            '{' => Root::get_path_from_local(&format!("code/{}", applet_path)).unwrap(),//Internal link
            _ => return Err("Internal link parsing error".to_owned())
        };
        let rust_path = Path::new(&rust_path);
        let rust_path_str = rust_path.to_str().unwrap();
        let applet_name = rust_path.file_name().unwrap().to_str().unwrap();
        let mut applet_camel_name = "".to_owned();
        let mut next_capital = true;
        for letter in applet_name.chars() {
            if letter == '_' {
                next_capital = true;
                continue;
            }
            if next_capital {
                applet_camel_name = format!("{}{}", applet_camel_name, letter.to_uppercase());
            } else {
                applet_camel_name = format!("{}{}", applet_camel_name, letter);
            }
            next_capital = false;
        }

        if !rust_path.exists() {
            let mut response = "".to_owned();
            println!("The path {} does not exist. Would you like to create it? [Y/n]", rust_path_str);
            let _=io::stdout().flush();
            io::stdin().read_line(&mut response).expect("Did not enter a correct string");
            if response == "N\n" || response == "n\n" {
                return Err(format!("The path {} does not exist.", rust_path_str));
            }

            let color = root.main_color.clone();
            let font_path = match &root.fonts {
                Some(f) => f[0].clone(),
                None => "FONT_PATH".to_owned(),
            };

            // Make the new rust applet
            let cargo_toml_text = format!("[package]
name = \"{applet_name}\"
version = \"0.1.0\"
edition = \"2018\"

[lib]
crate-type = [\"cdylib\", \"rlib\"]

[dependencies]
wikid_wasm = {{ git = \"https://github.com/jack-dinsmore/wikid_wasm.git\" }}
wasm-bindgen = \"0.2.92\"

[profile.release]
opt-level = \"s\"
");
            let main_text = format!("//Compile with `wasm-pack build --target web`
use wasm_bindgen::prelude::*;

use wikid_wasm::{{Applet, Style}};
use wikid_wasm::element::Element;

const WIDTH: u32 = {applet_width};
const HEIGHT: u32 = {applet_height};

#[wasm_bindgen]
pub struct {applet_camel_name} {{
    applet: Applet,
}}

#[wasm_bindgen]
impl {applet_camel_name} {{
    pub fn new(canvas: String) -> Self {{
        wikid_wasm::debug_panic();

        let mut style = Style::default(include_bytes!(\"{font_path}\"));
        style.set_color(\"{color}\");
        let applet = Applet::new(WIDTH, HEIGHT, canvas, style);

        Self {{
            applet,
        }}
    }}

    pub fn render(&mut self) {{
        self.applet.render(&self.get_elements());
    }}

    pub fn tick(&mut self) {{
        let elements = self.get_mut_elements();
        for callback in self.applet.tick(elements) {{
            match callback {{
                _ => ()
            }}
        }}
    }}

    pub fn mouse_button_down(&mut self, x: u32, y: u32) {{
        let elements = self.get_mut_elements();
        self.applet.mouse_button_down(x, y, elements);
    }}

    pub fn mouse_button_up(&mut self, x: u32, y: u32) {{
        let elements = self.get_mut_elements();
        self.applet.mouse_button_up(x, y, elements);
    }}

    pub fn mouse_move(&mut self, x: u32, y: u32) {{
        let elements = self.get_mut_elements();
        self.applet.mouse_move(x, y, elements);
    }}
}}

impl {applet_camel_name} {{
    fn get_elements(&self) -> Vec<*const dyn Element> {{
        vec![
        ]
    }}
    fn get_mut_elements(&mut self) -> Vec<*mut dyn Element> {{
        vec![
        ]
    }}
}}");

            let cargo_toml = format!("{}/Cargo.toml", rust_path_str);
            let src = format!("{}/src", rust_path_str);
            let main = format!("{}/src/lib.rs", rust_path_str);
            let _ = std::fs::create_dir(rust_path_str);
            let _ = std::fs::create_dir(src);
            std::fs::write(cargo_toml, cargo_toml_text).expect("Unable to write the Cargo.toml file");
            std::fs::write(main, main_text).expect("Unable to write the lib.rs file");
        }


        // Compile
        let command = format!("cd {} && wasm-pack build --target web", rust_path_str);
        let output = std::process::Command::new("sh").arg("-c").arg(command)
            .output();
        if let Err(e) = output {
            return Err(format!("Could not run compilation commands for applet `{}`\n{}", applet_path, e));
        }
        let output = output.unwrap();
        if !output.status.success() {
            return Err(format!("Compiler error while building applet `{}`\n{}", applet_path, String::from_utf8(output.stderr).unwrap()));
        }

        // Move to bin
        let bin_dir = Root::get_path_from_local(&format!("bin")).unwrap();
        let _ = std::fs::create_dir(&bin_dir);
        
        let bin_path = format!("{}/{}_bg.wasm", bin_dir, applet_name);
        let pkg_path = format!("{}/pkg/{}_bg.wasm", rust_path.to_str().unwrap(), applet_name);
        if let Err(_) = std::fs::rename(&pkg_path, &bin_path) {
            return Err("Could not move the compiled file to the binary directory".to_owned());
        }

        let bin_path = format!("{}/{}.js", bin_dir, applet_name);
        let pkg_path = format!("{}/pkg/{}.js", rust_path.to_str().unwrap(), applet_name);
        if let Err(_) = std::fs::rename(&pkg_path, &bin_path) {
            return Err("Could not move the compiled file to the binary directory".to_owned());
        }

        
        // Reset the link
        self.progress = 0;
        self.link_text = "".to_owned();
        self.link_type = '.';
        let bin_path = root.get_link_from_local(&format!("bin/{}.js", applet_name), public)?;
        let caption = self.display_text.clone();
        
        let res = Ok(format!("<center><canvas id=\"{applet_name}\"></canvas>
        <div class=\"caption\"><b>Applet:</b> {caption}</div></center>
<script type=\"module\">
    import init, {{{applet_camel_name}}} from \"{bin_path}\";
    await init();

    const {applet_name} = {applet_camel_name}.new(\"{applet_name}\");
    let last = Date.now();

    const renderLoop = () => {{
        let now = Date.now();
        requestAnimationFrame(renderLoop);
        if (now - last > 17) {{
            last = now;
            {applet_name}.tick();
            {applet_name}.render();
        }}
    }};
    document.getElementById(\"{applet_name}\").addEventListener(\"mousedown\", function(event) {{
        {applet_name}.mouse_button_down(event.offsetX, event.offsetY)
    }});
    document.getElementById(\"{applet_name}\").addEventListener(\"mouseup\", function(event) {{
        {applet_name}.mouse_button_up(event.offsetX, event.offsetY)
    }});
    document.getElementById(\"{applet_name}\").addEventListener(\"mousemove\", function(event) {{
        {applet_name}.mouse_move(event.offsetX, event.offsetY)
    }});
    requestAnimationFrame(renderLoop);

</script>"));
        
        self.display_text = "".to_owned();
        res
    }
    
    pub fn modifiers_is_latex(&self) -> bool {
        self.modifiers.is_latex()
    }
    
    pub fn unchecked_add(&mut self, c: char) {
        self.display_text.push(c)
    }
}
