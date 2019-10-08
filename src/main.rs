use cursive;

mod config;
use config::Configuration;

mod data;
use data::{Action, TreeType};

mod init;
use init::read_stacktraceflow_file;

use cursive::views::{ScrollView, IdView};

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
    type ScrollType = ScrollView<IdView<TreeType>>;
    let mut scroll_view = ScrollType::new(tree.with_id("tree"));
    scroll_view.set_scroll_y(false);
    scroll_view.set_scroll_x(true);
    siv.add_layer(scroll_view.with_id("scroll"));

    // Scroll on the x axis
    siv.call_on_id("tree", |tree: &mut TreeType| {
        tree.set_on_select(|s, row| {
            let x_position = s.call_on_id("tree", |tree: &mut TreeType| {
                match (tree.first_col(row), tree.item_width(row)) {
                    (Some(offset), Some(width)) => Some((offset, width)),
                    _ => None
                }
            });
            if let Some(Some((offset, width))) = x_position {
                s.call_on_id("scroll", |s: &mut ScrollType| {
                    let viewport = s.content_viewport();
                    if viewport.left() > offset {
                        s.set_offset((offset, viewport.top()));
                    } else if viewport.right() < offset + width {
                        s.set_offset((offset + width - viewport.width(), viewport.top()));
                    }
                });
            }
        });
    });

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
