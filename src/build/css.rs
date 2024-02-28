use std::str::FromStr;
use crate::root::Root;
use crate::constants::Color;
use crate::build::file_queue::FileQueue;

fn css_text(c: &str) -> String {
    let bw = Color::from_str(c).expect("Color was corrupted").bw().to_string();
    let light = Color::from_str(c).expect("Color was corrupted").light().to_string();
    let text = Color::from_str(c).expect("Color was corrupted").text().to_string();
    let bg = Color::from_str(c).expect("Color was corrupted").bg().to_string();
    let bg_image = match Root::summon().unwrap().bg_image {
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

    format!(
r"
body {{
    font-size: 16px;
    font-family: 'DM Sans', 'Nunito Sans', sans-serif;
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
    font-family: 'DM Sans', 'Nunito Sans', sans-serif;
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
    padding:bottom: 1em;
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
")
}

pub fn build_css(root: &Root, file_queue: &mut FileQueue) {
    for sec in &root.get_sections() {
        file_queue.add(format!("css/{}.css", sec.name), css_text(&sec.color));
    }
    file_queue.add("css/text.css".to_owned(), css_text(&root.main_color));
}
