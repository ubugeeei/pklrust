mod generator;

use clap::Parser;
use std::path::PathBuf;

/// pkl-gen-rust: Generate Rust source code from Pkl schema files.
#[derive(Parser, Debug)]
#[command(name = "pkl-gen-rust", version, about)]
struct Args {
    /// Input .pkl schema file(s).
    #[arg(required = true)]
    input: Vec<PathBuf>,

    /// Output directory for generated Rust files.
    #[arg(short, long, default_value = "src/generated")]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = run(&args) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(&args.output)?;

    for input in &args.input {
        let source = std::fs::read_to_string(input)?;
        let file_stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let generated = generator::generate(&source, file_stem)?;

        let output_path = args.output.join(format!("{file_stem}.rs"));
        std::fs::write(&output_path, generated)?;
        eprintln!("Generated: {}", output_path.display());
    }

    Ok(())
}
