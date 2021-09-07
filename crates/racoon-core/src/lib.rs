use std::collections::HashMap;

use indextree::{Arena, NodeId};

pub mod vecpointer;

pub struct RTag {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

impl RTag {
    pub fn new(name: String) -> RTag {
        RTag {
            name,
            attributes: HashMap::new(),
        }
    }
}

pub enum RNode {
    Tag(RTag),
    Text(String),
}

pub struct RDocument {
    pub arena: Arena<RNode>,
    pub root_key: NodeId,
}