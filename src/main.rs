use structopt::StructOpt;
use std::io::BufRead;

use cursive;
use cursive_tree_view;

use serde::{Serialize, Deserialize};

/// Analyze and pretty-print StackTraceFlow data from a Rust program
#[derive(StructOpt)]
struct Cli {
    /// Configuration file. Optional if all the required parameters are supplied on the command
    /// line
    #[structopt(parse(from_os_str), short, long)]
    config: Option<std::path::PathBuf>,

    /// File with the StackTraceFlow data
    #[structopt(parse(from_os_str), short, long)]
    file: Option<std::path::PathBuf>,

    /// Directory where the sources files are located
    #[structopt(parse(from_os_str), short, long)]
    dir: Option<std::path::PathBuf>,

    /// How deep should the printed tree be
    #[structopt(short = "N", long)]
    depth: Option<u16>,
}

#[derive(Serialize, Deserialize)]
struct FileConfig {
    file: Option<std::path::PathBuf>,

    /// Directory where the sources files are located
    dir: Option<std::path::PathBuf>,

    /// How deep should the printed tree be
    depth: Option<u16>,

    /// Cursor position
    selected: Option<usize>,

    /// Modifications to the tree (removals) performed by the user
    actions: Option<Vec<Action>>,
}

#[derive(Serialize, Deserialize)]
struct Action;

impl FileConfig {
    fn new() -> FileConfig {
        FileConfig{file: None, dir: None, depth: None, selected: None, actions: None}
    }
}

struct Configuration {
    config: std::path::PathBuf,
    file: std::path::PathBuf,
    dir: std::path::PathBuf,
    depth: u16,
    selected: usize,
    actions: Vec<Action>,
}

fn read_config(args: Cli) -> Configuration {
    use std::path::PathBuf;
    let mut file_config = FileConfig::new();
    let config_path = match args.config {
        Some(path) => {
            use toml;
            use std::io::Read;
            let mut file = std::fs::File::open(&path).expect(
                &format!("Could not open config file: {}", path.to_string_lossy()));
            let mut contents = "".to_string();
            file.read_to_string(&mut contents).expect(
                &format!("Could not read config file: {}", path.to_string_lossy()));
            file_config = toml::from_str(&contents).expect(
                &format!("Could not parse config file: {}", path.to_string_lossy()));
            path
        },
        None      => PathBuf::from("stacktraceflow.toml"),
    };

    use std::mem::replace;
    Configuration{
        config:     config_path,
        file:       args.file.or_else(|| file_config.file.clone()).expect(
            "You need to specify 'file' on the command line or in the config file"),
        dir:        args.dir.or_else(|| file_config.dir.clone()).expect(
            "You need to specify 'dir' on the command line or in the config file"),
        depth:      args.depth.or_else(|| file_config.depth).unwrap_or(10),
        selected:   file_config.selected.unwrap_or(1),
        actions:    file_config.actions.unwrap_or(Vec::<Action>::new()),
    }
}

fn main() {
    let configuration = read_config(Cli::from_args());

    let file = std::fs::File::open(&configuration.file).expect("Could not open file");
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
            if stack.len() <= configuration.depth as usize {
                let transposed = stack.last().unwrap().split('@').rfold(
                    "".to_string(),
                    |accum, next| { accum + " " + next.trim() },
                ).trim().to_string();
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
            if stack.len() < configuration.depth as usize {
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

    // [e]dit
    siv.add_global_callback('e', move |s| {
        s.call_on_id("tree", |tree: &mut cursive_tree_view::TreeView<String>| {
            if let Some(row) = tree.row() {
                let s = tree.borrow_item(row).unwrap();
                let mut split = s.split(':');
                let filename = split.next().unwrap();
                let line = split.next().unwrap().parse::<usize>().unwrap();

                use std::process::Command;
                Command::new("gnome-terminal")
                        .current_dir(&configuration.dir)
                        .arg("--")
                        .arg("vim")
                        .arg(&filename)
                        .arg(format!("+{}", line))
                        .status()
                        .expect("Failed to run command");

            }
        });
    });

    // [d]elete only this row without children
    siv.add_global_callback('d', move |s| {
        s.call_on_id("tree", |tree: &mut cursive_tree_view::TreeView<String>| {
            if let Some(row) = tree.row() {
                tree.extract_item(row);
            }
        });
    });

    // [D]elete all rows like this without their children
    siv.add_global_callback('D', move |s| {
        s.call_on_id("tree", |tree: &mut cursive_tree_view::TreeView<String>| {
            if let Some(row) = tree.row() {
                let mut row = row;
                if let Some(s) = tree.borrow_item(row) {
                    let s = s.clone();
                    for i in 0..tree.len() {
                        while let Some(x) = tree.borrow_item(i) {
                            if x != &s {
                                break;
                            }
                            if i <= row {
                                row -= 1;
                            }
                            tree.extract_item(i);
                        }
                    }
                    tree.set_selected_row(row);
                }
            }
        });
    });

    // [r]emove
    siv.add_global_callback('r', move |s| {
        s.call_on_id("tree", |tree: &mut cursive_tree_view::TreeView<String>| {
            if let Some(row) = tree.row() {
                tree.remove_item(row);
            }
        });
    });

    // [R]emove all rows like this and their children
    siv.add_global_callback('R', move |s| {
        s.call_on_id("tree", |tree: &mut cursive_tree_view::TreeView<String>| {
            if let Some(row) = tree.row() {
                let mut row = row;
                if let Some(s) = tree.borrow_item(row) {
                    let s = s.clone();
                    for i in 0..tree.len() {
                        while let Some(x) = tree.borrow_item(i) {
                            if x != &s {
                                break;
                            }
                            if let Some(v) = tree.remove_item(i) {
                                if i <= row {
                                    use std::cmp::min;
                                    row -= min(v.len(), row - i + 1);
                                }
                            }
                        }
                    }
                    tree.set_selected_row(row);
                }
            }
        });
    });

    // [q]uit
    siv.add_global_callback('q', |s| {
        use cursive::views::Dialog;
        s.add_layer(
            Dialog::text("Would you like to save the current configuration?")
            .title("Quitting")
            .button("Yes", |s| {
                s.quit();
            })
            .button("No", |s| { s.quit(); })
        );
    });
    siv.run();
}
