use std::io::BufRead;

use cursive;
use cursive_tree_view;

mod config;
use config::Configuration;

static mut CONFIGURATION: Option<Configuration> = None;

fn main() {
    let configuration;
    unsafe {
        CONFIGURATION = Some(Configuration::load());
        configuration = CONFIGURATION.as_ref().unwrap();
    }

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
    let dir = configuration.dir.clone();
    siv.add_global_callback('e', move |s| {
        s.call_on_id("tree", |tree: &mut cursive_tree_view::TreeView<String>| {
            if let Some(row) = tree.row() {
                let s = tree.borrow_item(row).unwrap();
                let mut split = s.split(':');
                let filename = split.next().unwrap();
                let line = split.next().unwrap().parse::<usize>().unwrap();

                use std::process::Command;
                Command::new("gnome-terminal")
                        .current_dir(&dir)
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
                unsafe {
                    CONFIGURATION.as_ref().unwrap().save();
                }
                s.quit();
            })
            .button("No", |s| { s.quit(); })
        );
    });
    siv.run();
}
