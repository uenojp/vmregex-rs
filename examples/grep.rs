use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Result;
use clap::Parser;
use vmregex::Regex;

#[derive(Parser)]
struct Cli {
    pattern: String,
    file: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let file = File::open(args.file)?;
    let reader = BufReader::new(file);
    let re = Regex::new(&args.pattern)?;

    for line in reader.lines() {
        let line = line?;
        let indices = line.char_indices().map(|(i, _)| i).collect::<Vec<_>>();
        for i in indices {
            if re.is_match(&line[i..])? {
                println!("{line}");
                break;
            }
        }
    }

    Ok(())
}
