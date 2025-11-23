use clap::Parser;
use megamaid::{run_command, Cli};

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run_command(cli.command, cli.config) {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}
