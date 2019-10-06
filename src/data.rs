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

impl Record {
    pub fn from_stacktraceflow_line(s: &String) -> Option<Self> {
        use regex::Regex;

        let re = Regex::new(r"(?x)
        ^
        (?P<function>[^@]+)
        \s@
        (?P<file>[^:]+)
        :
        (?P<line>\d+)
        :\d+:  # column
        \s\d+: # last line
        \d+    # last column
        $
        ").unwrap();
        let cap = re.captures(&s).unwrap();
        Some(Record{
            function: cap["function"].to_string(),
            file: cap["file"].to_string(),
            line: cap["line"].parse().ok()?,
        })
    }
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
