mod repl;
mod ui;

use clap::Parser;

#[derive(Parser)]
#[command(name = "MatyMemory", version, about = "A memory training game")]
struct Cli {
    /// Start in interactive REPL mode
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.interactive {
        repl::Repl::new().run();
    } else {
        println!("MatyMemory v{}", env!("CARGO_PKG_VERSION"));
        println!("Use -i to start interactive mode. Use --help for more info.");
    }
}
