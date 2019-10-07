use structopt::StructOpt;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::io::{Write, Read};
use regex::Regex;

use crate::data::Action;

#[derive(Clone)]
pub struct Configuration {
    pub config: std::path::PathBuf,
    pub file: std::path::PathBuf,
    pub dir: std::path::PathBuf,
    pub depth: u16,
    pub max_size: usize,
    pub selected: usize,
    pub actions: Vec<Action>,
    pub only: Vec<Regex>,
}

fn rpl<T: Default>(source: &mut T) -> T {
    use std::mem::replace;
    replace(source, T::default())
}

impl Configuration {
    pub fn load() -> Configuration {
        let mut args = Cli::from_args();
        let mut file_config = FileConfig::new();
        let config_path = match args.config {
            Some(path) => {
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

        let new_only_str = rpl(&mut file_config.only).unwrap_or(Vec::<String>::new());
        let mut new_only_rx: Vec<Regex> = new_only_str.iter().map(|s: &String| {
            Regex::new(s).expect(&format!("Cannot parse regex '{}", &s))
        }).collect();
        new_only_rx.append(&mut args.only);

        Configuration{
            config:     config_path,
            file:       args.file.or_else(|| file_config.file.clone()).expect(
                "You need to specify 'file' on the command line or in the config file"),
            dir:        args.dir.or_else(|| file_config.dir.clone()).expect(
                "You need to specify 'dir' on the command line or in the config file"),
            depth:      args.depth.or_else(|| file_config.depth).unwrap_or(
                if new_only_rx.is_empty() { 10 } else { 3 }
            ),
            max_size:   args.max_size.or_else(|| file_config.max_size).unwrap_or(10_000),
            selected:   file_config.selected.unwrap_or(1),
            actions:    file_config.actions.unwrap_or(Vec::new()),
            only:       new_only_rx,
        }
    }

    pub fn save(&self, selected: usize) {
        let path = &self.config;
        let mut file_config: FileConfig = self.clone().into();
        file_config.selected = Some(selected);
        let str_config = toml::to_string(&file_config).expect("Could not serialize Configuration");
        let mut file = std::fs::OpenOptions::new()
            .write(true).truncate(true).create(true).open(path)
            .expect(&format!("Could not open config file: {}", path.to_string_lossy()));
        let res = file.write(str_config.as_bytes());
        match res {
            Ok(n) if n == str_config.len() => { /* success */ },
            _ => panic!(
                "Failed to write config file {}. The file might be malformed",
                path.to_string_lossy()
            ),
        }
    }
}

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
    ///
    /// This is the depth of the entire tree if there are no 'only' patterns supplied.
    ///
    /// This is the depth of the subtrees (children) of the matching nodes if the 'only' patterns
    /// are supplied.
    #[structopt(short = "N", long)]
    depth: Option<u16>,

    /// Truncate the tree if it grows beyond this size
    #[structopt(short = "L", long)]
    max_size: Option<usize>,

    /// If any is specified, trim the tree to show only parents and children of the matching nodes
    #[structopt(long)]
    only: Vec<Regex>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileConfig {
    file: Option<std::path::PathBuf>,

    /// Directory where the sources files are located
    dir: Option<std::path::PathBuf>,

    /// How deep should the printed tree be
    depth: Option<u16>,

    /// Truncate the tree if it grows beyond this size
    max_size: Option<usize>,

    /// Cursor position
    selected: Option<usize>,

    /// If any is specified, trim the tree to show only parents and children of the nodes matching
    /// the regexes
    only: Option<Vec<String>>,

    /// Modifications to the tree (removals) performed by the user
    actions: Option<Vec<Action>>,
}

impl FileConfig {
    fn new() -> FileConfig {
        FileConfig{
            file: None,
            dir: None,
            depth: None,
            max_size: None,
            selected: None,
            actions: None,
            only: None
        }
    }
}

impl From<Configuration> for FileConfig {
    fn from(conf: Configuration) -> Self {
        let mut conf = conf;
        FileConfig {
            file: Some(rpl(&mut conf.file)),
            dir: Some(rpl(&mut conf.dir)),
            depth: Some(rpl(&mut conf.depth)),
            max_size: Some(rpl(&mut conf.max_size)),
            selected: Some(rpl(&mut conf.selected)),
            actions: if conf.actions.is_empty() { None } else { Some(rpl(&mut conf.actions)) },
            only: if conf.only.is_empty() { None } else {
                // Take conf's only (type: Vec<Regex>), map it into Vec<String>, and wrap in Some
                Some(rpl(&mut conf.only).iter().map(|r| r.to_string()).collect())
            }
        }
    }
}

