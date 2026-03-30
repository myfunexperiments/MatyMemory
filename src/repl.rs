use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Stylize,
    terminal,
};
use std::io::{self, Write};

use crate::ui;

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

pub struct Repl {
    input: String,
    ctrl_c_pending: bool,
    width: usize,
}

impl Repl {
    pub fn new() -> Self {
        let (w, _) = terminal::size().unwrap_or((80, 24));
        Self {
            input: String::new(),
            ctrl_c_pending: false,
            width: w as usize,
        }
    }

    pub fn run(&mut self) {
        terminal::enable_raw_mode().unwrap();
        let _guard = RawModeGuard;

        ui::draw_header(self.width);
        ui::draw_input_box(&self.input, self.width);

        loop {
            let ev = match event::read() {
                Ok(ev) => ev,
                Err(_) => break,
            };

            match ev {
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => {
                    if !self.handle_key(code, modifiers) {
                        break;
                    }
                }
                Event::Resize(w, _) => {
                    self.width = w as usize;
                    ui::clear_screen();
                    ui::draw_header(self.width);
                    ui::draw_input_box(&self.input, self.width);
                }
                _ => {}
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        // Ctrl+L — clear & redraw
        if code == KeyCode::Char('l') && modifiers.contains(KeyModifiers::CONTROL) {
            self.ctrl_c_pending = false;
            ui::clear_screen();
            ui::draw_header(self.width);
            ui::draw_input_box(&self.input, self.width);
            return true;
        }

        // Ctrl+C — twice to quit
        if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            if self.ctrl_c_pending {
                ui::erase_input_box();
                self.exit();
            }
            self.ctrl_c_pending = true;
            self.input.clear();
            ui::erase_input_box();
            let m = " ".repeat(ui::MARGIN);
            print!("{m}  {}\r\n\r\n", "Press Ctrl+C again to quit".dim());
            io::stdout().flush().unwrap();
            ui::draw_input_box(&self.input, self.width);
            return true;
        }

        self.ctrl_c_pending = false;

        match code {
            KeyCode::Enter => {
                let cmd = self.input.trim().to_string();
                self.input.clear();

                if cmd.is_empty() {
                    ui::redraw_content_line(&self.input, self.width);
                    return true;
                }

                ui::erase_input_box();
                self.exec_command(&cmd);
                ui::draw_input_box(&self.input, self.width);
            }
            KeyCode::Backspace => {
                if !self.input.is_empty() {
                    self.input.pop();
                    ui::redraw_content_line(&self.input, self.width);
                }
            }
            KeyCode::Char(c)
                if modifiers.is_empty() || modifiers == KeyModifiers::SHIFT =>
            {
                self.input.push(c);
                ui::redraw_content_line(&self.input, self.width);
            }
            _ => {}
        }

        true
    }

    fn exec_command(&self, input: &str) {
        let m = " ".repeat(ui::MARGIN);

        if !input.starts_with('/') {
            print!(
                "{m}  {} Type {} for available commands.\r\n\r\n",
                "Commands start with /".yellow(),
                "/help".cyan()
            );
            io::stdout().flush().unwrap();
            return;
        }

        let cmd = &input[1..];
        match cmd {
            "help" => {
                print!("{m}  {}\r\n\r\n", "Available commands:".bold());
                let cmds: &[(&str, &str)] = &[
                    ("/help", "Show this help message"),
                    ("/version", "Show version"),
                    ("/clear", "Clear the screen (Ctrl+L)"),
                    ("/exit", "Exit (also: /quit, Ctrl+C x2)"),
                ];
                for (name, desc) in cmds {
                    let pad = 12_usize.saturating_sub(name.len());
                    print!("{m}    {}{}{}\r\n", name.cyan(), " ".repeat(pad), desc.dim());
                }
                print!("\r\n");
            }
            "version" => {
                let v = format!("v{}", env!("CARGO_PKG_VERSION"));
                print!("{m}  MatyMemory {}\r\n\r\n", v.as_str().cyan());
            }
            "clear" => {
                ui::clear_screen();
                ui::draw_header(self.width);
            }
            "exit" | "quit" => {
                self.exit();
            }
            _ => {
                print!(
                    "{m}  {} '/{cmd}'. Type {} for available commands.\r\n\r\n",
                    "Unknown command:".red(),
                    "/help".cyan()
                );
            }
        }
        io::stdout().flush().unwrap();
    }

    fn exit(&self) -> ! {
        let m = " ".repeat(ui::MARGIN);
        print!("\r\n{m}  {}\r\n\r\n", "Goodbye!".cyan());
        io::stdout().flush().unwrap();
        terminal::disable_raw_mode().unwrap();
        std::process::exit(0);
    }
}
