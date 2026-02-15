use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cfdl", version, about = "CFDL v0.1 compiler CLI")]
struct Cli {
    /// Emit diagnostics as JSON to stdout (on failure)
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Compile {
        model_root: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Validate {
        model_root: PathBuf,
    },
    Parse {
        model_root: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Compile { model_root, out } => match cfdl_compile::compile_to_file(&model_root, &out) {
            Ok(()) => Ok(()),
            Err(diags) => {
                if cli.json {
                    // ONLY JSON to stdout
                    print!("{}", serde_json::to_string_pretty(&diags)?);
                } else {
                    eprintln!("Compilation failed with {} diagnostic(s).", diags.len());
                    for d in &diags {
                        eprintln!("{}[{}] {}", d.severity.to_uppercase(), d.code, d.message);
                    }
                }
                std::process::exit(1);
            }
        },
        Command::Validate { model_root } => match cfdl_compile::validate_only(&model_root) {
            Ok(()) => Ok(()),
            Err(diags) => {
                if cli.json {
                    print!("{}", serde_json::to_string_pretty(&diags)?);
                } else {
                    eprintln!("Validation failed with {} diagnostic(s).", diags.len());
                    for d in &diags {
                        eprintln!("{}[{}] {}", d.severity.to_uppercase(), d.code, d.message);
                    }
                }
                std::process::exit(1);
            }
        },
        Command::Parse { model_root: _ } => Ok(()),
    }
}