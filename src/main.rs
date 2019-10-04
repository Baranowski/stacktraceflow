use structopt::StructOpt;
use std::io::BufRead;

use atty;

mod output;

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
    let mut output: output::Output = if atty::is(atty::Stream::Stdout) { output::Tui::new() } else { output::Text::new() };

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("+") {
            stack.push(line[1..].to_owned());
            let transposed = stack.last().unwrap().split('@').rfold(
                "".to_string(),
                |accum, next| { accum + " " + next },
            );
            if stack.len() <= args.depth as usize {
                output.add_node(&transposed, stack.len()-1);
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
            output.close_level();
        } else {
            panic!("Line '{}' starts with neither '+' nor '-' in line {}", line, counter);
        }
        counter += 1;
    }

    output.finalize();
}
