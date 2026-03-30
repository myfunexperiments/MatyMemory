use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};
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

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

fn repl() {
    println!("MatyMemory v{} - Interactive Mode", env!("CARGO_PKG_VERSION"));
    println!("Type 'help' for available commands.\n");

    terminal::enable_raw_mode().unwrap();

    let mut input = String::new();
    let mut ctrl_c_pending = false;

    print_prompt();

    loop {
        let ev = match event::read() {
            Ok(ev) => ev,
            Err(_) => break,
        };

        let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = ev
        else {
            continue;
        };

        // Ctrl+L: clear screen
        if code == KeyCode::Char('l') && modifiers.contains(KeyModifiers::CONTROL) {
            ctrl_c_pending = false;
            clear_screen();
            print_prompt();
            print!("{input}");
            io::stdout().flush().unwrap();
            continue;
        }

        // Ctrl+C: twice to quit
        if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            if ctrl_c_pending {
                print!("\r\n");
                cleanup_and_exit("Goodbye!");
            }
            ctrl_c_pending = true;
            input.clear();
            print!("\r\n(Press Ctrl+C again to quit)\r\n");
            print_prompt();
            continue;
        }

        // Any other key resets the Ctrl+C state
        ctrl_c_pending = false;

        match code {
            KeyCode::Enter => {
                print!("\r\n");

                let cmd = input.trim().to_string();
                input.clear();

                if cmd.is_empty() {
                    print_prompt();
                    continue;
                }

                match cmd.as_str() {
                    "help" => print_help(),
                    "version" => {
                        print!("MatyMemory v{}\r\n", env!("CARGO_PKG_VERSION"));
                    }
                    "clear" => {
                        clear_screen();
                        print_prompt();
                        continue;
                    }
                    "exit" | "quit" => {
                        cleanup_and_exit("Goodbye!");
                    }
                    _ => {
                        print!("Unknown command: '{cmd}'. Type 'help' for available commands.\r\n");
                    }
                }
                print_prompt();
            }
            KeyCode::Backspace => {
                if !input.is_empty() {
                    input.pop();
                    // Move cursor back, overwrite with space, move back again
                    print!("\x08 \x08");
                    io::stdout().flush().unwrap();
                }
            }
            KeyCode::Char(c) => {
                input.push(c);
                print!("{c}");
                io::stdout().flush().unwrap();
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode().unwrap();
}

fn print_prompt() {
    print!(">> ");
    io::stdout().flush().unwrap();
}

fn print_help() {
    print!("Available commands:\r\n");
    print!("  help      Show this help message\r\n");
    print!("  version   Show version\r\n");
    print!("  clear     Clear the screen (also: Ctrl+L)\r\n");
    print!("  exit      Exit the REPL (also: quit, Ctrl+C x2)\r\n");
}

fn cleanup_and_exit(msg: &str) -> ! {
    terminal::disable_raw_mode().unwrap();
    println!("{msg}");
    std::process::exit(0);
}
