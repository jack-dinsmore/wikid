use std::str::FromStr;
use crate::root::Root;
use crate::constants::Color;
use crate::build::file_queue::FileQueue;

fn css_text(c: &str, public: bool) -> String {
    let bw = Color::from_str(c).expect("Color was corrupted").bw().to_string();
    let light = Color::from_str(c).expect("Color was corrupted").light().to_string();
    let text = Color::from_str(c).expect("Color was corrupted").text().to_string();
    let bg = Color::from_str(c).expect("Color was corrupted").bg().to_string();
    let root = Root::summon().unwrap();
    let mut preamble = "".to_owned();
    let bg_image = match root.bg_image {
        Some(_) => {
            format!(r"background-image: url(../css/background_image.png);
    background-repeat: no-repeat;
    background-attachment: fixed;
    background-size: 100% 100%;")
        },
        None => {
            format!("background: {};", bg)
        }
    };
    let font_family = match &root.fonts {
        Some(fonts) => {
            let mut out = "".to_owned();
            let mut font_index = 0;
            for item in fonts {
                if item.ends_with(".ttf") {
                    font_index += 1;
                    // This is a path. Make the font css entry
                    let name = format!("font{font_index}");
                    let item_link = root.get_link_from_local(item, public).unwrap();
                    preamble = format!("{preamble}
@font-face {{
    font-family: \"{name}\";
    src: url(\"{item_link}\");
}}");
                    out = format!("{} '{}', ", out, name);
                } else {
                    out = format!("{} '{}', ", out, item);
                }
            }
            out = format!("{} 'sans-serif'", out);
            out
        },
        None => {
            "'DM Sans', 'Nunito Sans', sans-serif".to_owned()
        }
    };
    let font_size = root.font_size;

    format!(
r"
{preamble}
body {{
    font-size: {font_size}px;
    font-family: {font_family};
    color: {text};
    {bg_image}
}}

h1, h2, h3, h4, h5 {{
    color: {text};
}}

a {{
    color: {c};
    font-weight: bold;
    text-decoration: none;
}}

a:visited {{
    color: {c};
    font-weight: normal;
}}

a:hover {{
    background: {c};
    color: {bw};
}}

#content {{
    padding-top: 100px;
    width: 50em;
    margin: auto;
    line-height: 150%;
    padding-bottom: 3em;
}}

#footer {{
    border-top: solid {text} 2px;
    padding-top: 1em;
    padding-bottom: 1em;
    padding-left: 15em;
    padding-right: 15em;
    margin: auto;
    width: 50em;
}}

.footnote {{
    font-size: 14px;
}}

.collapsible {{
    background-color: {bg};
    color: white;
    cursor: pointer;
    width: 100%;
    padding-left: 5px;
    line-height: 100%;
    border: none;
    text-align: left;
    outline: none;
    font-family: {font_family};
}}
  
.collapsible:hover {{
    background-color: {light};
}}

.section {{
    padding-left: 18px;
    padding-top: 0px;
    overflow: hidden;
    transition: max-height 0.2s ease-out;
}}

.caption {{
    font-size: 14px;
    padding-top: 1em;
    padding-bottom: 1em;
}}

.tooltip .tooltiptext {{
    visibility: hidden;
    width: 120px;
    background-color: black;
    color: #fff;
    text-align: left;
    border-radius: 6px;
    padding: 15px;

    /* Position the tooltip */
    position: absolute;
    z-index: 1;
}}

.eq {{
    display: flex;
    flex-direction: row;
    align-items: center;
}}
.eqtext {{
    width: 95%;
}}

.eqnum {{
    width: 5%;
    text-align: right;
}}

")
}

pub fn build_css(root: &Root, file_queue: &mut FileQueue, public: bool) {
    for sec in &root.get_sections() {
        file_queue.add(format!("css/{}.css", sec.name), css_text(&sec.color, public));
    }
    file_queue.add("css/text.css".to_owned(), css_text(&root.main_color, public));
}
