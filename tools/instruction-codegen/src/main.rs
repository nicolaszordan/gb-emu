#[allow(dead_code)] // allowing dead_code while we wait for parameterized code-gen
mod gb;

use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Generate Rust code from GB instruction definitions")]
struct Args {
    /// Path to the input JSON file containing instruction definitions
    #[arg(short, long)]
    input: PathBuf,

    /// Path to the output .rs file to generate
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    println!(
        "generating instruction code from {} to {}",
        args.input.display(),
        args.output.display()
    );

    println!(
        "reading instruction definitions from {}...",
        args.input.display()
    );
    let input = fs::read_to_string(&args.input).expect("Failed to read input file");

    println!("generating code...");
    let output = gb::generate_code(&input).expect("Failed to generate code");

    println!("writing output to {}...", args.output.display());
    let mut file = fs::File::create(&args.output).expect("Failed to create output file");
    file.write_all(output.as_bytes())
        .expect("Failed to write output");
}
