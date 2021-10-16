use crate::root::Root;
use crate::build::file_queue::FileQueue;

pub fn build_css(root: &Root, file_queue: &mut FileQueue) {
    for sec in &root.sections {
        file_queue.add(format!("css/{}.css", sec.name), format!(
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
", sec.color)
);
    }
}
