use structopt::StructOpt;
use std::io::BufRead;

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
        } else {
            panic!("Line '{}' starts with neither '+' nor '-' in line {}", line, counter);
        }
        counter += 1;
    }
}
