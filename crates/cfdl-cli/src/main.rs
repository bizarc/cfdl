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
        #[arg(long)]
        packs: Option<PathBuf>,
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
        #[arg(long)]
        packs: Option<PathBuf>,
        /// Domain pack name for post-engine metrics (e.g. "cre", "opco")
        #[arg(long)]
        pack: Option<String>,
    },
    Pack {
        #[command(subcommand)]
        command: PackCommand,
    },
}

#[derive(Subcommand, Debug)]
enum PackCommand {
    List {
        #[arg(long)]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Compile {
            model_root,
            out,
            packs,
        } => {
            let options = cfdl_compile::CompileOptions {
                packs_dir: packs.or_else(default_packs_dir),
            };
            match cfdl_compile::compile_to_file_with_options(&model_root, &out, &options) {
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
        Command::Parse { model_root } => {
            let model_file = model_root.join("model.cfdl");
            let source = match std::fs::read_to_string(&model_file) {
                Ok(source) => source,
                Err(err) => {
                    eprintln!("error: cannot read '{}': {err}", model_file.display());
                    std::process::exit(1);
                }
            };
            let (tokens, lex_diags) = cfdl_lexer::lex(&source);
            if !lex_diags.is_empty() {
                let rendered: Vec<serde_json::Value> = lex_diags
                    .iter()
                    .map(|d| {
                        serde_json::json!({
                            "code": d.code,
                            "message": d.message,
                            "span": d.span,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&rendered)?);
                std::process::exit(1);
            }
            let parse_result = cfdl_parser::parse("model.cfdl", &source, &tokens);
            if !parse_result.diagnostics.is_empty() {
                let rendered: Vec<serde_json::Value> = parse_result
                    .diagnostics
                    .iter()
                    .map(|d| {
                        serde_json::json!({
                            "code": d.code,
                            "message": d.message,
                            "file": d.file,
                            "span": d.span,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&rendered)?);
                std::process::exit(1);
            }
            let ast = parse_result.ast.expect("no diagnostics implies AST");
            println!("{}", serde_json::to_string_pretty(&ast)?);
            Ok(())
        }
        Command::Run {
            ir_json_path,
            out,
            config,
            rate,
            as_of,
            packs,
            pack,
        } => {
            let mut registry: Option<cfdl_pack::PackRegistry> = None;
            if let Some(pack_dir) = packs.or_else(default_packs_dir) {
                match cfdl_pack::PackRegistry::load_from_dir(&pack_dir) {
                    Ok(loaded) => registry = Some(loaded),
                    Err(err) => {
                        emit_run_failure(
                            cli.json,
                            vec![RunDiagnostic {
                                code: "E4004_MISSING_PACK".to_string(),
                                severity: "error".to_string(),
                                message: err.message,
                                file: None,
                                span: None,
                                path: None,
                                hint: None,
                                notes: vec![format!("pack root: {}", pack_dir.display())],
                            }],
                        )?;
                        std::process::exit(1);
                    }
                }
            }
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

            let mut results = match cfdl_engine::run_from_file(&ir_json_path, run_config) {
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
            if let Some(pack_name) = &pack {
                let specs = registry
                    .as_ref()
                    .map(|reg| reg.metric_specs(pack_name))
                    .unwrap_or_default();
                results.domain_metrics = cfdl_metrics::compute(pack_name, &specs, &results);
            }
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
        Command::Pack { command } => match command {
            PackCommand::List { path } => match cfdl_pack::PackRegistry::load_from_dir(&path) {
                Ok(registry) => {
                    if cli.json {
                        let entries: Vec<serde_json::Value> = registry
                            .list()
                            .into_iter()
                            .map(|pack| {
                                serde_json::json!({
                                    "name": pack.manifest.name,
                                    "version": pack.manifest.version
                                })
                            })
                            .collect();
                        print!("{}", serde_json::to_string_pretty(&entries)?);
                    } else {
                        for pack in registry.list() {
                            println!("{} {}", pack.manifest.name, pack.manifest.version);
                        }
                    }
                    Ok(())
                }
                Err(err) => {
                    if cli.json {
                        print!(
                            "{}",
                            serde_json::to_string_pretty(&vec![RunDiagnostic {
                                code: "E4004_MISSING_PACK".to_string(),
                                severity: "error".to_string(),
                                message: err.message,
                                file: None,
                                span: None,
                                path: None,
                                hint: None,
                                notes: vec![],
                            }])?
                        );
                    } else {
                        eprintln!("ERROR[E4004_MISSING_PACK] {}", err.message);
                    }
                    std::process::exit(1);
                }
            },
        },
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

fn default_packs_dir() -> Option<PathBuf> {
    let candidate = PathBuf::from("packs");
    if candidate.is_dir() {
        Some(candidate)
    } else {
        None
    }
}
