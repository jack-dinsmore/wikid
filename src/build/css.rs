use crate::root::Root;
use crate::build::file_queue::FileQueue;

const DEFAULT_COLOR: &'static str = "#000";

fn css_text(c: &str) -> String {
    format!(
"body {{
    font-size: 12 px;
    font-family: sans-serif;
}}

h1, h2, h3, h4, h5 {{
    color: {}
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
", c)
}

pub fn build_css(root: &Root, file_queue: &mut FileQueue) {
    for sec in &root.sections {
        file_queue.add(format!("css/{}.css", sec.name), css_text(&sec.color));
    }
    file_queue.add("css/main.css".to_owned(), css_text(DEFAULT_COLOR));
}
