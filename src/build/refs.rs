use std::collections::HashMap;
use crate::root::Root;

pub struct RefMap {
    posts: HashMap<String, (String, String)>,// interior_label, (exterior_label, link)
    secs: HashMap<String, (String, u32, String)>,// interior_label, (exterior_label, number, link)
    eqs: HashMap<String, (u32, u32, String)>,// interior_Label, (section number, eqn number, link)
    projects: HashMap<String, (String, String)>,// interior_label, (external_label, link)
    vocab: HashMap<Vec<String>, String>,// display names, link
}

impl RefMap {
    pub fn new (root: &Root) -> RefMap {
        let posts = HashMap::new();
        let secs = HashMap::new();
        let eqs = HashMap::new();
        let projects = HashMap::new();
        let vocab = HashMap::new();

        // Load glossary

        // Load section exterior names

        // Load posts names and equations

        // Load project labels from .wikid and from github link

        RefMap { posts, secs, eqs, projects, vocab }
    }
}
