use std::io::BufRead;

use crate::data::{Record, TreeType};
use crate::config::Configuration;

struct Node {
    /// Original line from the StackTraceFlow file, tripped of the initial '+' or '-' sign
    orig_line: String,

    /// Record into which the line above was parsed
    record: Record,

    /// row (i.e., an id) of the associated line in the view. Set iff the node is being displayed
    view_row: Option<usize>,
}

fn add_line(
    configuration: &Configuration,
    tree: &mut TreeType,
    stack: &mut Vec<Node>,
    line: &str,
) {
    let mut view_row = None;
    let record = Record::from_stacktraceflow_line(line).expect(
        &format!("Failed to parse stacktraceflow line: {}", &line));
    if stack.len() < configuration.depth as usize {

        view_row = tree.insert_item(
            record.clone(),
            cursive_tree_view::Placement::LastChild,
            stack.last().map_or(0, |node| node.view_row.unwrap_or(0)),
        );
    }
    stack.push(Node{
        orig_line: line.to_string(),
        record: record,
        view_row: view_row,
    });
}

fn del_line(configuration: &Configuration, stack: &mut Vec<Node>, line: &str, counter: usize) {
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
    let mut stack: Vec<Node> = Vec::new();

    let file = std::fs::File::open(&configuration.file).expect("Could not open file");
    let reader = std::io::BufReader::new(file);
    let mut counter: usize = 1;

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("+") {
            add_line(configuration, tree, &mut stack, &line[1..]);
        } else if line.starts_with("-") {
            del_line(configuration, &mut stack, &line[1..], counter);
        } else {
            panic!("Line '{}' starts with neither '+' nor '-' in line {}", line, counter);
        }
        counter += 1;
    }
}
