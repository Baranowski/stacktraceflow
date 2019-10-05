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

    /// Directory where the sources files are located
    #[structopt(parse(from_os_str), short, long, default_value = "./")]
    dir: std::path::PathBuf,

    /// How deep should the printed tree be
    #[structopt(short = "N", long, default_value = "10")]
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
                let transposed = stack.last().unwrap().split('@').rfold(
                    "".to_string(),
                    |accum, next| { accum + " " + next },
                );
                println!(
                    "{:indent$}{name}", "",
                    name=transposed,
                    indent=(stack.len()-1)*4
                );
                let new_row_opt = tree.insert_item(
                    transposed,
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

    use cursive::traits::Identifiable;
    let mut siv = cursive::Cursive::default();
    siv.add_layer(tree.with_id("tree"));
    siv.add_global_callback('e', move |s| {
        s.call_on_id("tree", |tree: &mut cursive_tree_view::TreeView<String>| {
            if let Some(row) = tree.row() {
                let s = tree.borrow_item(row).unwrap();
                let mut split = s.split(':');
                let filename = split.next().unwrap();
                let line = split.next().unwrap().parse::<usize>().unwrap();

                use std::process::Command;
                Command::new("gnome-terminal")
                        .current_dir(&args.dir)
                        .arg("--")
                        .arg("vim")
                        .arg(&filename)
                        .arg(format!("+{}", line))
                        .status()
                        .expect("Failed to run command");

            }
        });
    });
    siv.run();
}
