use clap::Parser;
#[derive(Parser)]
#[command(
    name = "preflight",
    version,
    about = "Preverjanje razvojnega okolja pred delavnico"
)]
struct Args {}
fn main() {
    Args::parse();
    println!(
        "Preflight {} — preverjalnik je v pripravi.",
        preflight_core::app_info().version
    );
}
