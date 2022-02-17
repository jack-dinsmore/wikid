use std::str::FromStr;
use crate::root::Root;
use crate::constants::Color;
use crate::build::file_queue::FileQueue;

const DEFAULT_COLOR: &'static str = "#000000;";

fn css_text(c: &str) -> String {
    let bw_color = Color::from_str(c).expect("Color was corrupted").bw();
    let light_color = Color::from_str(c).expect("Color was corrupted").light();

    format!(
r"body {{
    font-size: 16px;
    font-family: 'DM Sans', sans-serif;
}}

h1, h2, h3, h4, h5 {{
    color: #000
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
    font-weight: normal;
}}

#content {{
    padding-top: 100px;
    width: 50em;
    margin: auto;
    line-height: 150%;
    padding-bottom: 3em;
}}

#footer {{
    border-top: solid black 2px;
    padding-top: 1em;
    padding-bottom: 1em;
    padding-left: 15em;
    padding-right: 15em;
    margin: auto;
    width: 50em;
}}

.collapsible {{
    background-color: #fff;
    color: white;
    cursor: pointer;
    width: 100%;
    padding-left: 5px;
    line-height: 100%;
    border: none;
    text-align: left;
    outline: none;
    font-family: 'DM Sans', sans-serif;
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
", c=c, bw=bw_color.to_string(), light=light_color.to_string())
}

pub fn build_css(root: &Root, file_queue: &mut FileQueue) {
    for sec in &root.sections {
        file_queue.add(format!("css/{}.css", sec.name), css_text(&sec.color));
    }
    file_queue.add("css/main.css".to_owned(), css_text(DEFAULT_COLOR));
}
