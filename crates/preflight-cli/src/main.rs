use clap::{Parser, Subcommand, ValueEnum};
use preflight_core::{
    engine::{check_profile, Platform},
    process::SystemRunner,
    Error, Profile, ReasonCode, Status,
};
use std::{io::Read, path::PathBuf, process::ExitCode};
#[derive(Parser)]
#[command(
    name = "preflight",
    version,
    about = "Preverjanje razvojnega okolja pred delavnico"
)]
struct Args {
    #[command(subcommand)]
    command: Action,
}
#[derive(Subcommand)]
enum Action {
    /// Preveri lokalno okolje po YAML profilu
    Check {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
    },
}
#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Json,
    Text,
}
fn read_profile(path: PathBuf) -> Result<Profile, Error> {
    let file = std::fs::File::open(path).map_err(|_| Error::new(ReasonCode::ProfileUnreadable))?;
    let mut yaml = String::new();
    file.take((preflight_core::model::MAX_PROFILE_BYTES + 1) as u64)
        .read_to_string(&mut yaml)
        .map_err(|_| Error::new(ReasonCode::ProfileUnreadable))?;
    Profile::parse(&yaml)
}
#[tokio::main]
async fn main() -> ExitCode {
    let Args {
        command: Action::Check { profile, format },
    } = Args::parse();
    let profile = match read_profile(profile) {
        Ok(p) => p,
        Err(e) => {
            match format {
                Format::Json => println!(
                    "{}",
                    serde_json::to_string(&e).expect("static error serialization")
                ),
                Format::Text => eprintln!("{}", e),
            }
            return ExitCode::from(2);
        }
    };
    let report = check_profile(&profile, &SystemRunner, Platform::current()).await;
    match format {
        Format::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("report serialization")
        ),
        Format::Text => {
            println!("Preflight — lokalno preverjanje\n");
            for r in &report.results {
                let label = match r.status {
                    Status::Pass => "V REDU",
                    Status::Fail => "NAPAKA",
                    Status::Warning => "OPOZORILO",
                    Status::Unknown => "NEZNANO",
                };
                println!(
                    "[{}] {}{}: {}",
                    label,
                    r.check_id,
                    r.installed_version
                        .as_ref()
                        .map(|v| format!(" ({v})"))
                        .unwrap_or_default(),
                    r.message_sl
                );
                if r.status != Status::Pass {
                    println!("  {}", r.suggestion_sl)
                }
            }
            println!(
                "\n{} Rezultati ostanejo na tvojem računalniku.",
                if report.summary.ready {
                    "Obvezne zahteve so izpolnjene."
                } else {
                    "Okolje še ni pripravljeno."
                }
            );
        }
    }
    ExitCode::from(report.exit_code())
}
