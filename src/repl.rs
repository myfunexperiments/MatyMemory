use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Stylize,
    terminal,
};
use std::io::{self, Write};

use crate::ui;

const COMMANDS: &[(&str, &str)] = &[
    ("/clear", "Clear the screen (Ctrl+L)"),
    ("/exit", "Exit (also: /quit, Ctrl+C x2)"),
    ("/help", "Show this help message"),
    ("/quit", "Quit the app"),
    ("/version", "Show version"),
];

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

const MAX_VISIBLE: usize = 3;

pub struct Repl {
    input: String,
    ctrl_c_pending: bool,
    width: usize,
    suggestions: Vec<usize>,
    selected: Option<usize>,
    scroll_offset: usize,
    history: Vec<String>,
    history_index: Option<usize>,
    saved_input: String,
}

impl Repl {
    pub fn new() -> Self {
        let (w, _) = terminal::size().unwrap_or((80, 24));
        Self {
            input: String::new(),
            ctrl_c_pending: false,
            width: w as usize,
            suggestions: Vec::new(),
            selected: None,
            scroll_offset: 0,
            history: Vec::new(),
            history_index: None,
            saved_input: String::new(),
        }
    }

    pub fn run(&mut self) {
        terminal::enable_raw_mode().unwrap();
        let _guard = RawModeGuard;

        ui::draw_header(self.width);
        ui::draw_input_box(&self.input, &[], None, self.width);

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
                    self.suggestions.clear();
                    self.selected = None;
                    self.scroll_offset = 0;
                    ui::clear_screen();
                    ui::draw_header(self.width);
                    ui::draw_input_box(&self.input, &[], None, self.width);
                }
                _ => {}
            }
        }
    }

    fn visible_suggestions(&self) -> Vec<(&str, &str)> {
        let end = (self.scroll_offset + MAX_VISIBLE).min(self.suggestions.len());
        self.suggestions[self.scroll_offset..end]
            .iter()
            .map(|&i| COMMANDS[i])
            .collect()
    }

    fn visible_selected(&self) -> Option<usize> {
        self.selected.map(|s| s - self.scroll_offset)
    }

    fn update_suggestions(&mut self) {
        if self.input.starts_with('/') {
            self.suggestions = COMMANDS
                .iter()
                .enumerate()
                .filter(|(_, (name, _))| name.starts_with(self.input.as_str()))
                .map(|(i, _)| i)
                .collect();
            // Hide when input exactly matches the only result
            if self.suggestions.len() == 1 && COMMANDS[self.suggestions[0]].0 == self.input {
                self.suggestions.clear();
            }
        } else {
            self.suggestions.clear();
        }
        self.selected = None;
        self.scroll_offset = 0;
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        // Ctrl+L — clear & redraw
        if code == KeyCode::Char('l') && modifiers.contains(KeyModifiers::CONTROL) {
            self.ctrl_c_pending = false;
            self.suggestions.clear();
            self.selected = None;
            self.scroll_offset = 0;
            ui::clear_screen();
            ui::draw_header(self.width);
            ui::draw_input_box(&self.input, &[], None, self.width);
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
            self.suggestions.clear();
            self.selected = None;
            self.scroll_offset = 0;
            ui::erase_input_box();
            let m = " ".repeat(ui::MARGIN);
            print!("{m}  {}\r\n\r\n", "Press Ctrl+C again to quit".dim());
            io::stdout().flush().unwrap();
            ui::draw_input_box(&self.input, &[], None, self.width);
            return true;
        }

        self.ctrl_c_pending = false;

        match code {
            KeyCode::Enter => {
                if let Some(i) = self.selected {
                    // Accept suggestion into input without executing
                    self.input = COMMANDS[self.suggestions[i]].0.to_string();
                    self.suggestions.clear();
                    self.selected = None;
                    self.scroll_offset = 0;
                    ui::redraw_input_area(&self.input, &[], None, self.width);
                    return true;
                }

                let cmd = self.input.trim().to_string();
                self.input.clear();
                self.suggestions.clear();
                self.selected = None;
                self.scroll_offset = 0;
                self.history_index = None;
                self.saved_input.clear();

                if cmd.is_empty() {
                    ui::redraw_input_area(&self.input, &[], None, self.width);
                    return true;
                }

                // Add to history (avoid consecutive duplicates)
                if self.history.last().map_or(true, |last| last != &cmd) {
                    self.history.push(cmd.clone());
                }

                ui::erase_input_box();
                self.exec_command(&cmd);
                ui::draw_input_box(&self.input, &[], None, self.width);
            }
            KeyCode::Backspace => {
                if !self.input.is_empty() {
                    self.input.pop();
                    self.history_index = None;
                    self.saved_input.clear();
                    self.update_suggestions();
                    let items = self.visible_suggestions();
                    ui::redraw_input_area(&self.input, &items, self.visible_selected(), self.width);
                }
            }
            KeyCode::Tab => {
                if !self.suggestions.is_empty() {
                    let idx = self.selected.unwrap_or(0);
                    self.input = COMMANDS[self.suggestions[idx]].0.to_string();
                    self.suggestions.clear();
                    self.selected = None;
                    self.scroll_offset = 0;
                    ui::redraw_input_area(&self.input, &[], None, self.width);
                }
            }
            KeyCode::Down => {
                if !self.suggestions.is_empty() {
                    let new_sel = match self.selected {
                        None => 0,
                        Some(i) => (i + 1).min(self.suggestions.len() - 1),
                    };
                    self.selected = Some(new_sel);
                    // Scroll down if selected goes past visible window
                    if new_sel >= self.scroll_offset + MAX_VISIBLE {
                        self.scroll_offset = new_sel + 1 - MAX_VISIBLE;
                    }
                    let items = self.visible_suggestions();
                    ui::redraw_input_area(&self.input, &items, self.visible_selected(), self.width);
                } else if self.history_index.is_some() {
                    // Navigate forward in history
                    let idx = self.history_index.unwrap();
                    if idx + 1 < self.history.len() {
                        self.history_index = Some(idx + 1);
                        self.input = self.history[idx + 1].clone();
                    } else {
                        // Back to saved input
                        self.history_index = None;
                        self.input = self.saved_input.clone();
                        self.saved_input.clear();
                    }
                    self.update_suggestions();
                    let items = self.visible_suggestions();
                    ui::redraw_input_area(&self.input, &items, self.visible_selected(), self.width);
                }
            }
            KeyCode::Up => {
                if !self.suggestions.is_empty() {
                    self.selected = match self.selected {
                        None | Some(0) => None,
                        Some(i) => Some(i - 1),
                    };
                    // Scroll up if selected goes above visible window
                    if let Some(s) = self.selected {
                        if s < self.scroll_offset {
                            self.scroll_offset = s;
                        }
                    }
                    let items = self.visible_suggestions();
                    ui::redraw_input_area(&self.input, &items, self.visible_selected(), self.width);
                } else if !self.history.is_empty() {
                    // Navigate backward in history
                    let new_idx = match self.history_index {
                        None => {
                            self.saved_input = self.input.clone();
                            self.history.len() - 1
                        }
                        Some(0) => 0,
                        Some(i) => i - 1,
                    };
                    self.history_index = Some(new_idx);
                    self.input = self.history[new_idx].clone();
                    self.update_suggestions();
                    let items = self.visible_suggestions();
                    ui::redraw_input_area(&self.input, &items, self.visible_selected(), self.width);
                }
            }
            KeyCode::Esc => {
                if !self.suggestions.is_empty() {
                    self.suggestions.clear();
                    self.selected = None;
                    self.scroll_offset = 0;
                    ui::redraw_input_area(&self.input, &[], None, self.width);
                }
            }
            KeyCode::Char(c)
                if modifiers.is_empty() || modifiers == KeyModifiers::SHIFT =>
            {
                self.input.push(c);
                self.history_index = None;
                self.saved_input.clear();
                self.update_suggestions();
                let items = self.visible_suggestions();
                ui::redraw_input_area(&self.input, &items, self.visible_selected(), self.width);
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
                for (name, desc) in COMMANDS {
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
