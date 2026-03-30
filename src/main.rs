use clap::Parser;
use std::io::{self, Write};

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
        repl();
    } else {
        println!("MatyMemory v{}", env!("CARGO_PKG_VERSION"));
        println!("Use -i to start interactive mode. Use --help for more info.");
    }
}

fn repl() {
    println!("MatyMemory v{} - Interactive Mode", env!("CARGO_PKG_VERSION"));
    println!("Type 'help' for available commands.\n");

    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!(">> ");
        io::stdout().flush().unwrap();

        input.clear();
        if stdin.read_line(&mut input).unwrap() == 0 {
            break;
        }

        let cmd = input.trim();
        if cmd.is_empty() {
            continue;
        }

        match cmd {
            "help" => print_help(),
            "version" => println!("MatyMemory v{}", env!("CARGO_PKG_VERSION")),
            "clear" => print!("\x1B[2J\x1B[1;1H"),
            "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Unknown command: '{}'. Type 'help' for available commands.", cmd),
        }
    }
}

fn print_help() {
    println!("Available commands:");
    println!("  help      Show this help message");
    println!("  version   Show version");
    println!("  clear     Clear the screen");
    println!("  exit      Exit the REPL (also: quit)");
}
