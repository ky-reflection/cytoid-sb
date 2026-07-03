mod input;
mod pipeline;

use clap::{Parser, Subcommand};
use cytoid_sb_diag::SbError;
use miette::Result as MietteResult;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "cytoid-sb",
    about = "Compile and validate Cytoid storyboard authoring inputs",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging (RUST_LOG still applies).
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Validate a storyboard JSON file or level directory.
    Check {
        /// Input `.json` file or level directory containing storyboard JSON.
        input: camino::Utf8PathBuf,
    },

    /// Compile authoring input to storyboard.generated.json.
    Compile {
        /// Input `.json`, `.lua`, or level directory.
        input: camino::Utf8PathBuf,

        /// Output path (default: `<input-dir>/storyboard.generated.json`).
        #[arg(short, long)]
        output: Option<camino::Utf8PathBuf>,
    },

    /// Re-check input when files change.
    Watch {
        input: camino::Utf8PathBuf,

        #[arg(long, default_value = "500")]
        debounce_ms: u64,
    },
}

fn init_tracing(verbose: bool) {
    let default = if verbose { "debug" } else { "info" };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn main() -> MietteResult<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command {
        Commands::Check { input } => run_check(&input),
        Commands::Compile { input, output } => run_compile(&input, output.as_deref()),
        Commands::Watch { input, debounce_ms } => run_watch(&input, debounce_ms),
    }
}

fn run_check(input: &camino::Utf8Path) -> MietteResult<()> {
    let report = pipeline::check_path(input)?;
    if report.is_ok() {
        tracing::info!(
            path = %input,
            objects = report.object_count(),
            summary = %report.summary.format_line(),
            "check passed"
        );
        println!("ok: {} ({})", input, report.summary.format_line());
        Ok(())
    } else {
        for issue in &report.validation.issues {
            eprintln!("[{}] {}", issue.code, issue.message);
        }
        Err(SbError::Validation {
            count: report.validation.issues.len(),
        }
        .into())
    }
}

fn run_compile(input: &camino::Utf8Path, output: Option<&camino::Utf8Path>) -> MietteResult<()> {
    let out_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| input::default_generated_path(input));

    let report = pipeline::compile_path(input, &out_path)?;
    tracing::info!(
        input = %input,
        output = %out_path,
        summary = %report.summary.format_line(),
        "compile finished"
    );
    println!("wrote {} ({})", out_path, report.summary.format_line());
    Ok(())
}

fn run_watch(input: &camino::Utf8Path, debounce_ms: u64) -> MietteResult<()> {
    println!(
        "watching {} (debounce {}ms); Ctrl+C to stop",
        input, debounce_ms
    );
    let mut last_mtime = file_mtime(input);
    loop {
        std::thread::sleep(Duration::from_millis(debounce_ms));
        let mtime = file_mtime(input);
        if mtime != last_mtime {
            last_mtime = mtime;
            match run_check(input) {
                Ok(()) => println!("watch: ok"),
                Err(err) => eprintln!("watch: {err}"),
            }
        }
    }
}

fn file_mtime(path: &camino::Utf8Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path.as_std_path())
        .ok()
        .and_then(|m| m.modified().ok())
}
