//! Useful datatypes

use serde::{Serialize, Deserialize};

use cursive_tree_view;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", content = "args")]
pub enum Action {
    Recursive(Record),
    Delete(Record),
}

/// A record to be shown in the tree
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub struct Record {
    pub function: String,
    pub file: String,
    pub line: usize,
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{file}:{line}    [{function}]",
            file=&self.file,
            line=&self.line,
            function=&self.function,
        )
    }
}

pub type TreeType = cursive_tree_view::TreeView<Record>;
