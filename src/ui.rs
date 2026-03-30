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
    let char_count = input.chars().count();
    if char_count > cw {
        let skip = char_count - cw;
        let byte_idx = input.char_indices().nth(skip).map(|(i, _)| i).unwrap_or(0);
        &input[byte_idx..]
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

fn render_suggestion_line(name: &str, desc: &str, is_selected: bool, term_width: usize) {
    let m = margin();
    let cw = content_width(term_width);
    let name_len = name.chars().count();
    let min_gap = 2;
    let desc_budget = cw.saturating_sub(name_len + min_gap);
    let desc_display: String = desc.chars().take(desc_budget).collect();
    let desc_len = desc_display.chars().count();
    let gap = cw.saturating_sub(name_len + desc_len);

    if is_selected {
        let line = format!("{}{}{}", name, " ".repeat(gap), desc_display);
        print!("{m}  {}", line.on_dark_cyan().white());
    } else {
        print!("{m}  {}{}{}", name.cyan(), " ".repeat(gap), desc_display.dim());
    }
}

fn render_box_and_suggestions(
    input: &str,
    suggestions: &[(&str, &str)],
    selected: Option<usize>,
    term_width: usize,
) {
    let m = margin();
    let bw = box_width(term_width);
    let cw = content_width(term_width);

    // Top border
    let dash_count = bw.saturating_sub(7);
    let top_rest = format!(" {}╮", "─".repeat(dash_count));
    print!(
        "{m}{}{}{}\r\n",
        "╭─ ".dark_grey(),
        ">>".bold().cyan(),
        top_rest.as_str().dark_grey()
    );

    // Content line
    let display = display_input(input, term_width);
    let dlen = display.chars().count();
    let padding = cw.saturating_sub(dlen);
    print!(
        "{m}{} {}{} {}\r\n",
        "│".dark_grey(),
        display,
        " ".repeat(padding),
        "│".dark_grey()
    );

    // Bottom border (no trailing \r\n)
    let bottom = format!("╰{}╯", "─".repeat(bw.saturating_sub(2)));
    print!("{m}{}", bottom.as_str().dark_grey());

    // Suggestion lines
    if !suggestions.is_empty() {
        for (i, (name, desc)) in suggestions.iter().enumerate() {
            print!("\r\n");
            render_suggestion_line(name, desc, selected == Some(i), term_width);
        }
    }

    // Move cursor back to content line
    let lines_below = if suggestions.is_empty() { 1 } else { 1 + suggestions.len() as u16 };
    let col = (MARGIN + 2 + dlen) as u16;
    execute!(io::stdout(), cursor::MoveUp(lines_below), cursor::MoveToColumn(col)).unwrap();
    io::stdout().flush().unwrap();
}

pub fn draw_input_box(
    input: &str,
    suggestions: &[(&str, &str)],
    selected: Option<usize>,
    term_width: usize,
) {
    render_box_and_suggestions(input, suggestions, selected, term_width);
}

pub fn redraw_input_area(
    input: &str,
    suggestions: &[(&str, &str)],
    selected: Option<usize>,
    term_width: usize,
) {
    // Cursor is on content line; move to top border and clear everything below
    execute!(
        io::stdout(),
        cursor::MoveUp(1),
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::FromCursorDown),
    )
    .unwrap();
    render_box_and_suggestions(input, suggestions, selected, term_width);
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
