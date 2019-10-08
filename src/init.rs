use std::io::BufRead;
use std::collections::HashMap;
use regex::Regex;

use crate::data::{Record, TreeType};
use crate::config::Configuration;

struct Node {
    /// Original line from the StackTraceFlow file, tripped of the initial '+' or '-' sign
    orig_line: String,

    /// row (i.e., an id) of the associated line in the view. Set iff the node is being displayed
    view_row: Option<usize>,

    /// Did the current node match one of the 'only' patterns
    matched_an_only: bool,
}

struct StackTraceFlowParser {
    re:      Regex,
    matches: HashMap<String, bool>,
}

impl StackTraceFlowParser {
    fn new() -> Self {
        StackTraceFlowParser{
            re: Regex::new(r"(?x)
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
            ").expect("Failed to compile the parser's regex"),
            matches: HashMap::new(),
        }
    }

    fn parse(&self, line: &str) -> Record {
        let cap = self.re.captures(line).expect(
            &format!("Failed to capture based on regex from the line '{}'", line));
        Record{
            function: cap["function"].to_string(),
            file: cap["file"].to_string(),
            line: cap["line"].parse().expect(
                &format!("Failed to parse line number in '{}'", line)),
        }
    }
}

fn add_line_with_full_tree(
    parser: &mut StackTraceFlowParser,
    configuration: &Configuration,
    tree: &mut TreeType,
    stack: &mut Vec<Node>,
    line: &str,
) {
    let mut view_row: Option<usize> = None;
    if stack.len() < configuration.depth as usize {

        if tree.len() < configuration.max_size {
            view_row = Some(tree.insert_item(
                parser.parse(line),
                cursive_tree_view::Placement::LastChild,
                stack.last().map_or(0, |node| node.view_row.unwrap_or(0)),
            ).unwrap());
        }
    }
    stack.push(Node{
        orig_line: line.to_string(),
        view_row: view_row,
        matched_an_only: false,
    });
}

fn matches_an_only(
    parser: &mut StackTraceFlowParser,
    line: &str,
    onlys: &Vec<Regex>,
) -> bool {
    match parser.matches.get(line) {
        Some(b) => *b,
        None => {
            let value = onlys.iter().any(|re| re.is_match(&parser.parse(line).to_string()));
            parser.matches.insert(line.to_string(), value);
            value
        }
    }
}

/// We decided to add a node to a tree. Make sure that all its ancestors have been added first
fn add_current_path(
    parser: &mut StackTraceFlowParser,
    stack: &mut Vec<Node>,
    tree: &mut TreeType,
) {
    let mut previous_row: usize = 0;
    for i in stack.iter_mut() {
        if let None = i.view_row {
            i.view_row = tree.insert_item(
                parser.parse(&i.orig_line),
                cursive_tree_view::Placement::LastChild,
                previous_row,
            );
        }
        previous_row = i.view_row.unwrap();
    }
}

fn add_line_with_only(
    parser: &mut StackTraceFlowParser,
    configuration: &Configuration,
    tree: &mut TreeType,
    stack: &mut Vec<Node>,
    line: &str,
) {
    let matched = matches_an_only(parser, line, &configuration.only);
    if tree.len() >= configuration.max_size {
        stack.push(Node{
            orig_line: line.to_string(),
            view_row: None,
            matched_an_only: matched,
        });
        return;
    }

    let mut view_row = None;
    if matched {
        // The current entry matches one of the 'only' patterns
        add_current_path(parser, stack, tree);
        view_row = Some(tree.insert_item(
            parser.parse(line),
            cursive_tree_view::Placement::LastChild,
            stack.last().as_ref().unwrap().view_row.unwrap(),
        ).unwrap());
    } else {
        let recent_ancestors = stack.iter().rev().take(configuration.depth as usize);
        let mut recent_ancestor_matched = false;
        for ref item in recent_ancestors {
            if item.matched_an_only {
                recent_ancestor_matched = true;
                break;
            }
        }
        if recent_ancestor_matched {
            add_current_path(parser, stack, tree);
            view_row = Some(tree.insert_item(
                parser.parse(line),
                cursive_tree_view::Placement::LastChild,
                stack.last().as_ref().unwrap().view_row.unwrap(),
            ).unwrap());
        }
    }
    stack.push(Node{
        orig_line: line.to_string(),
        view_row: view_row,
        matched_an_only: matched,
    });
}

fn del_line(_configuration: &Configuration, stack: &mut Vec<Node>, line: &str, counter: usize) {
    let topmost_line = &stack.last().expect(
        "Read a '-' StackTraceFlow line with an empty stack"
    ).orig_line;
    if topmost_line != &line {
        panic!(
            "StackTraceFlow line '{}' does not match top of the stack '{}' in line {}",
            line, topmost_line, counter,
        );
    }
    stack.pop();
}

pub fn read_stacktraceflow_file(configuration: &Configuration, tree: &mut TreeType) {
    let mut parser = StackTraceFlowParser::new();
    let mut stack: Vec<Node> = Vec::new();

    let file = std::fs::File::open(&configuration.file).expect("Could not open file");
    let reader = std::io::BufReader::new(file);
    let mut counter: usize = 1;
    let add_fn = if configuration.only.is_empty() {
        add_line_with_full_tree
    } else {
        add_line_with_only
    };

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("+") {
            add_fn(&mut parser, configuration, tree, &mut stack, &line[1..]);
        } else if line.starts_with("-") {
            del_line(configuration, &mut stack, &line[1..], counter);
        } else {
            panic!("Line '{}' starts with neither '+' nor '-' in line {}", line, counter);
        }
        counter += 1;
        if counter%100000 == 0 {
            println!("{}", counter);
        }
    }
}
