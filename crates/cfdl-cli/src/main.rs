use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;
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
    Run {
        ir_json_path: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value_t = 0.0)]
        rate: f64,
        #[arg(long)]
        as_of: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Compile { model_root, out } => {
            match cfdl_compile::compile_to_file(&model_root, &out) {
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
            }
        }
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
        Command::Run {
            ir_json_path,
            out,
            config,
            rate,
            as_of,
        } => {
            let parsed_as_of = if let Some(as_of) = as_of {
                match cfdl_engine::Date::parse(&as_of) {
                    Ok(date) => Some(date),
                    Err(_) => {
                        emit_run_failure(
                            cli.json,
                            vec![RunDiagnostic {
                                code: "E5002_IR_SCHEMA_VALIDATION_FAILED".to_string(),
                                severity: "error".to_string(),
                                message: format!(
                                    "Invalid --as-of value '{as_of}', expected YYYY-MM-DD."
                                ),
                                file: None,
                                span: None,
                                path: None,
                                hint: Some("Use ISO date format like 2026-12-31.".to_string()),
                                notes: vec![],
                            }],
                        )?;
                        std::process::exit(1);
                    }
                }
            } else {
                None
            };

            let run_config = if let Some(config_path) = config {
                match cfdl_engine::run_config_from_json_file(&config_path, rate, parsed_as_of) {
                    Ok(config) => config,
                    Err(err) => {
                        emit_run_failure(
                            cli.json,
                            vec![RunDiagnostic {
                                code: "E5002_IR_SCHEMA_VALIDATION_FAILED".to_string(),
                                severity: "error".to_string(),
                                message: format!(
                                    "Failed to load run config '{}': {err}",
                                    config_path.display()
                                ),
                                file: Some(config_path.to_string_lossy().to_string()),
                                span: None,
                                path: None,
                                hint: Some(
                                    "Ensure run.json is valid JSON and matches run config schema."
                                        .to_string(),
                                ),
                                notes: vec![],
                            }],
                        )?;
                        std::process::exit(1);
                    }
                }
            } else {
                cfdl_engine::RunConfig {
                    discount_rate: rate,
                    as_of: parsed_as_of,
                    ..Default::default()
                }
            };

            let results = match cfdl_engine::run_from_file(&ir_json_path, run_config) {
                Ok(results) => results,
                Err(err) => {
                    emit_run_failure(
                        cli.json,
                        vec![RunDiagnostic {
                            code: "E5002_IR_SCHEMA_VALIDATION_FAILED".to_string(),
                            severity: "error".to_string(),
                            message: format!(
                                "Run failed while reading IR '{}': {err}",
                                ir_json_path.display()
                            ),
                            file: Some(ir_json_path.to_string_lossy().to_string()),
                            span: None,
                            path: None,
                            hint: None,
                            notes: vec![],
                        }],
                    )?;
                    std::process::exit(1);
                }
            };
            let json = match serde_json::to_string_pretty(&results) {
                Ok(json) => json,
                Err(err) => {
                    emit_run_failure(
                        cli.json,
                        vec![RunDiagnostic {
                            code: "E5003_IR_EMIT_FAILED".to_string(),
                            severity: "error".to_string(),
                            message: format!("Failed to serialize results JSON: {err}"),
                            file: None,
                            span: None,
                            path: None,
                            hint: None,
                            notes: vec![],
                        }],
                    )?;
                    std::process::exit(1);
                }
            };
            if let Err(err) = std::fs::write(&out, json) {
                emit_run_failure(
                    cli.json,
                    vec![RunDiagnostic {
                        code: "E5003_IR_EMIT_FAILED".to_string(),
                        severity: "error".to_string(),
                        message: format!("Failed to write results file '{}': {err}", out.display()),
                        file: Some(out.to_string_lossy().to_string()),
                        span: None,
                        path: None,
                        hint: None,
                        notes: vec![],
                    }],
                )?;
                std::process::exit(1);
            }
            Ok(())
        }
    }
}

#[derive(Debug, Serialize)]
struct RunDiagnostic {
    code: String,
    severity: String,
    message: String,
    file: Option<String>,
    span: Option<RunSpan>,
    path: Option<String>,
    hint: Option<String>,
    notes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RunSpan {
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
}

fn emit_run_failure(json_mode: bool, diags: Vec<RunDiagnostic>) -> Result<()> {
    if json_mode {
        print!("{}", serde_json::to_string_pretty(&diags)?);
    } else {
        eprintln!("Run failed with {} diagnostic(s).", diags.len());
        for d in &diags {
            eprintln!("{}[{}] {}", d.severity.to_uppercase(), d.code, d.message);
        }
    }
    Ok(())
}
