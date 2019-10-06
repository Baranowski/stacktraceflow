use structopt::StructOpt;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::io::{Write, Read};

#[derive(Clone)]
pub struct Configuration {
    pub config: std::path::PathBuf,
    pub file: std::path::PathBuf,
    pub dir: std::path::PathBuf,
    pub depth: u16,
    pub selected: usize,
    pub actions: Vec<Action>,
}

fn rpl<T: Default>(source: &mut T) -> T {
    use std::mem::replace;
    replace(source, T::default())
}

impl Configuration {
    pub fn load() -> Configuration {
        let args = Cli::from_args();
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

    pub fn save(&self) {
        let path = &self.config;
        let file_config: FileConfig = self.clone().into();
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Action;

impl FileConfig {
    fn new() -> FileConfig {
        FileConfig{file: None, dir: None, depth: None, selected: None, actions: None}
    }
}

impl From<Configuration> for FileConfig {
    fn from(conf: Configuration) -> Self {
        let mut conf = conf;
        FileConfig {
            file: Some(rpl(&mut conf.file)),
            dir: Some(rpl(&mut conf.dir)),
            depth: Some(rpl(&mut conf.depth)),
            selected: Some(rpl(&mut conf.selected)),
            actions: Some(rpl(&mut conf.actions)),
        }
    }
}

