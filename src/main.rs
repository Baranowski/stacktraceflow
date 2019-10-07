use std::io::BufRead;

use cursive;

mod config;
use config::Configuration;

mod data;
use data::{Action, Record, TreeType};

static mut CONFIGURATION: Option<Configuration> = None;

fn perform_action(act: &Action, tree: &mut TreeType) {
    match *act {
        Action::Delete(ref record) => {
            if let Some(mut row) = tree.row() {
                for i in 0..tree.len() {
                    while let Some(x) = tree.borrow_item(i) {
                        if x != record {
                            break;
                        }
                        if i <= row && row > 0 {
                            row -= 1;
                        }
                        tree.extract_item(i);
                    }
                }
                tree.set_selected_row(row);
            }
        },
        Action::Recursive(ref record) => {
            if let Some(mut row) = tree.row() {
                for i in 0..tree.len() {
                    while let Some(x) = tree.borrow_item(i) {
                        if x != record {
                            break;
                        }
                        if let Some(v) = tree.remove_item(i) {
                            if i <= row {
                                use std::cmp::min;
                                row -= min(min(v.len(), row - i + 1), row);
                            }
                        }
                    }
                }
                tree.set_selected_row(row);
            }
        },
    }
}

fn add_action(act: Action) {
    unsafe {
        CONFIGURATION.as_mut().unwrap().actions.push(act);
    }
}

fn read_stacktraceflow_file(configuration: &Configuration, tree: &mut TreeType) {
    let file = std::fs::File::open(&configuration.file).expect("Could not open file");
    let reader = std::io::BufReader::new(file);
    let mut stack: Vec<String> = Vec::new();
    let mut counter: u32 = 1;
    let mut tree_stack : Vec<usize> = Vec::new();
    tree_stack.push(0);

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("+") {
            let entry_id = line[1..].to_string();
            let entry = Record::from_stacktraceflow_line(&entry_id).expect(
                &format!("Failed to parse stacktraceflow line: {}", &entry_id));
            stack.push(entry_id);
            if stack.len() <= configuration.depth as usize {
                let new_row_opt = tree.insert_item(
                    entry,
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
}

fn main() {
    let configuration;
    unsafe {
        CONFIGURATION = Some(Configuration::load());
        configuration = CONFIGURATION.as_ref().unwrap();
    }

    let mut tree = TreeType::new();
    read_stacktraceflow_file(configuration, &mut tree);

    for act in &configuration.actions {
        perform_action(&act, &mut tree);
    }

    use cursive::traits::Identifiable;
    let mut siv = cursive::Cursive::default();
    siv.add_layer(tree.with_id("tree"));

    // [e]dit
    let dir = configuration.dir.clone();
    siv.add_global_callback('e', move |s| {
        s.call_on_id("tree", |tree: &mut TreeType| {
            if let Some(row) = tree.row() {
                let record = tree.borrow_item(row).unwrap();

                use std::process::Command;
                Command::new("gnome-terminal")
                        .current_dir(&dir)
                        .arg("--")
                        .arg("vim")
                        .arg(&record.file)
                        .arg(format!("+{}", record.line))
                        .status()
                        .expect("Failed to run command");

            }
        });
    });

    // [d]elete only this row without children
    siv.add_global_callback('d', move |s| {
        s.call_on_id("tree", |tree: &mut TreeType| {
            if let Some(row) = tree.row() {
                tree.extract_item(row);
            }
        });
    });

    // [D]elete all rows like this without their children
    siv.add_global_callback('D', move |s| {
        s.call_on_id("tree", |tree: &mut TreeType| {
            if let Some(row) = tree.row() {
                if let Some(s) = tree.borrow_item(row) {
                    let action = Action::Delete(s.clone());
                    perform_action(&action, tree);
                    add_action(action);
                }
            }
        });
    });

    // [r]ecursively remove
    siv.add_global_callback('r', move |s| {
        s.call_on_id("tree", |tree: &mut TreeType| {
            if let Some(row) = tree.row() {
                tree.remove_item(row);
            }
        });
    });

    // [R]ecursively remove all rows like this and their children
    siv.add_global_callback('R', move |s| {
        s.call_on_id("tree", |tree: &mut TreeType| {
            if let Some(row) = tree.row() {
                if let Some(s) = tree.borrow_item(row) {
                    let action = Action::Recursive(s.clone());
                    perform_action(&action, tree);
                    add_action(action);
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
                let row = s.call_on_id("tree", |tree: &mut TreeType| {
                    tree.row().unwrap_or(0)
                }).unwrap_or(0);
                unsafe {
                    CONFIGURATION.as_ref().expect(
                        "The CONFIGURATION object died before saving. Please report an issue"
                    ).save(row);
                }
                s.quit();
            })
            .button("No", |s| { s.quit(); })
        );
    });
    siv.run();
}
