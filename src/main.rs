use structopt::StructOpt;
use std::io::BufRead;

use cursive;
use cursive_tree_view;

/// Analyze and pretty-print StackTraceFlow data from a Rust program
#[derive(StructOpt)]
struct Cli {
    /// File with the StackTraceFlow data
    #[structopt(parse(from_os_str), short, long)]
    file: std::path::PathBuf,

    /// How deep should the printed tree be
    #[structopt(short, long, default_value = "3")]
    depth: u16,
}

fn main() {
    let args = Cli::from_args();
    let file = std::fs::File::open(&args.file).expect("Could not open file");
    let reader = std::io::BufReader::new(file);
    let mut stack: Vec<String> = Vec::new();
    let mut counter: u32 = 1;
    let mut tree = cursive_tree_view::TreeView::<String>::new();
    let mut tree_stack : Vec<usize> = Vec::new();
    tree_stack.push(0);

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("+") {
            stack.push(line[1..].to_owned());
            if stack.len() <= args.depth as usize {
                println!(
                    "{:indent$}{name}", "",
                    name=stack.last().unwrap(),
                    indent=(stack.len()-1)*4
                );
                let new_row_opt = tree.insert_item(
                    stack.last().unwrap().clone(),
                    cursive_tree_view::Placement::LastChild,
                    *tree_stack.last().unwrap(),
                );
                let new_row = match new_row_opt {
                    Some(x) => x,
                    None => !0,
                };
                tree_stack.push(new_row);
            }
        } else if line.starts_with("-") {
            if stack.last() != Some(&line[1..].to_owned()) {
                panic!(
                    "StackTraceFlow line '{}' does not match top of the stack '{}' in line {}",
                    line,
                    stack.last().unwrap(),
                    counter,
                );
            }
            stack.pop();
            if stack.len() < args.depth as usize {
                tree_stack.pop();
            }
        } else {
            panic!("Line '{}' starts with neither '+' nor '-' in line {}", line, counter);
        }
        counter += 1;
    }

    let mut siv = cursive::Cursive::default();
    siv.add_layer(tree);
    siv.run();
}
