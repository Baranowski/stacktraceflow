use cursive;
use cursive_tree_view;

pub trait Output {
    fn add_level(&self, s: &String, depth: u16) -> ();
    fn close_level(&self) -> ();
    fn finalize(&self) -> ();
}

pub struct Text;

impl Output for Text {
    fn add_level(&self, s: &String, depth: u16) {
        println!("{:indent$}{name}", "", name=s, indent=depth*4);
    }

    fn close_level(&self) {}

    fn finalize(&self) {}
}

pub struct Tui {
    tree: cursive_tree_view::TreeView<String>,
    tree_stack: Vec<usize>,
}

impl Tui {
    pub fn new() -> Tui {
        let tree = cursive_tree_view::TreeView::<String>::new();
        let stack = Vec::<usize>::new([0]);
        Tui{tree, stack}
    }
}

impl Output for Tui {
    fn add_level(&self, s: &String, depth: u16) {
        let new_row_opt = self.tree.insert_item(
            s,
            cursive_tree_view::Placement::LastChild,
            *self.tree_stack.last().unwrap(),
        );
        let row_id = match new_row_opt {
            Some(x) => x,
            None => !0,
        };
        self.tree_stack.push(row_id);
    }

    fn close_level(&self) {
        self.tree_stack.pop();
    }

    fn finalize(&self) {
        let mut siv = cursive::Cursive::default();
        siv.add_layer(self.tree);
        siv.run();
    }
}

