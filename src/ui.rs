use crossterm::{cursor, execute, style::Stylize, terminal::{self, ClearType}};
use std::io::{self, Write};

pub const MARGIN: usize = 2;

const LOGO_TOP: &[&str] = &[
    "   ╔╦╗╔═╗╔╦╗╦ ╦",
    "   ║║║╠═╣ ║ ╚╦╝",
    "   ╩ ╩╩ ╩ ╩  ╩ ",
];

const LOGO_BOTTOM: &[&str] = &[
    "╔╦╗╔═╗╔╦╗╔═╗╦═╗╦ ╦",
    "║║║║╣ ║║║║ ║╠╦╝╚╦╝",
    "╩ ╩╚═╝╩ ╩╚═╝╩╚═ ╩ ",
];

fn margin() -> String {
    " ".repeat(MARGIN)
}

fn box_width(term_width: usize) -> usize {
    term_width.saturating_sub(MARGIN * 2).max(20)
}

fn content_width(term_width: usize) -> usize {
    box_width(term_width).saturating_sub(4)
}

pub fn display_input(input: &str, term_width: usize) -> &str {
    let cw = content_width(term_width);
    if input.len() > cw {
        &input[input.len() - cw..]
    } else {
        input
    }
}

pub fn draw_header(term_width: usize) {
    let m = margin();

    print!("\r\n");
    for line in LOGO_TOP {
        print!("{m}  {}\r\n", line.bold().cyan());
    }
    for line in LOGO_BOTTOM {
        print!("{m}  {}\r\n", line.cyan());
    }
    print!("\r\n");

    let info = format!("v{} · Interactive Mode", env!("CARGO_PKG_VERSION"));
    print!("{m}  {}\r\n", info.as_str().dark_cyan());
    print!("{m}  {}\r\n", "Type /help for commands".dim());

    let bw = box_width(term_width);
    let sep = "─".repeat(bw.saturating_sub(4));
    print!("\r\n{m}  {}\r\n\r\n", sep.as_str().dark_grey());

    io::stdout().flush().unwrap();
}

pub fn draw_input_box(input: &str, term_width: usize) {
    let m = margin();
    let bw = box_width(term_width);
    let cw = content_width(term_width);

    let dash_count = bw.saturating_sub(7);
    let top_rest = format!(" {}╮", "─".repeat(dash_count));
    print!(
        "{m}{}{}{}\r\n",
        "╭─ ".dark_grey(),
        ">>".bold().cyan(),
        top_rest.as_str().dark_grey()
    );

    let display = display_input(input, term_width);
    let padding = cw.saturating_sub(display.len());
    print!(
        "{m}{} {}{} {}\r\n",
        "│".dark_grey(),
        display,
        " ".repeat(padding),
        "│".dark_grey()
    );

    let bottom = format!("╰{}╯", "─".repeat(bw.saturating_sub(2)));
    print!("{m}{}", bottom.as_str().dark_grey());

    let col = (MARGIN + 2 + display.len()) as u16;
    execute!(io::stdout(), cursor::MoveUp(1), cursor::MoveToColumn(col)).unwrap();
    io::stdout().flush().unwrap();
}

pub fn redraw_content_line(input: &str, term_width: usize) {
    let m = margin();
    let cw = content_width(term_width);
    let display = display_input(input, term_width);
    let padding = cw.saturating_sub(display.len());

    print!(
        "\r{m}{} {}{} {}",
        "│".dark_grey(),
        display,
        " ".repeat(padding),
        "│".dark_grey()
    );

    let col = (MARGIN + 2 + display.len()) as u16;
    execute!(io::stdout(), cursor::MoveToColumn(col)).unwrap();
    io::stdout().flush().unwrap();
}

pub fn erase_input_box() {
    execute!(
        io::stdout(),
        cursor::MoveUp(1),
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::FromCursorDown)
    )
    .unwrap();
}

pub fn clear_screen() {
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();
}
