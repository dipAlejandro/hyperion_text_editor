use crate::search::SearchState;
use crossterm::{
    cursor,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::fmt::Write as _;
use std::io::Write;

pub fn render_line_number<W: Write>(stdout: &mut W, line_number: usize, row: u16, width: usize) {
    write!(stdout, "{}", cursor::MoveTo(0, row)).unwrap();
    write!(stdout, "{}", SetForegroundColor(Color::Cyan)).unwrap();
    write!(stdout, "{:>width$} ", line_number, width = width - 1).unwrap();
    write!(stdout, "{}", ResetColor).unwrap();
}

pub fn render_line_content<W: Write>(
    stdout: &mut W,
    line: &str,
    line_idx: usize,
    start_col: usize,
    search: &SearchState,
    is_current_line: bool,
) {
    let start_byte = char_to_byte_idx(line, start_col);
    let visible_line = &line[start_byte..];
    let line_bg = is_current_line.then_some(Color::DarkGrey);

    if search.is_active() {
        let highlighted = highlight_matches(line, line_idx, start_col, search, line_bg);
        write!(stdout, "{}", highlighted).unwrap();
    } else {
        if let Some(color) = line_bg {
            write!(
                stdout,
                "{}{}{}",
                SetBackgroundColor(color),
                visible_line,
                ResetColor
            )
            .unwrap();
        } else {
            write!(stdout, "{}", visible_line).unwrap();
        }
    }
}

fn highlight_matches(
    line: &str,
    line_idx: usize,
    start_col: usize,
    search: &SearchState,
    line_bg: Option<Color>,
) -> String {
    let start_byte = char_to_byte_idx(line, start_col);
    let visible_line = &line[start_byte..];
    let mut highlighted_line = String::new();
    let mut current_pos = 0;

    if let Some(color) = line_bg {
        write!(highlighted_line, "{}", SetBackgroundColor(color)).unwrap();
    }

    for m in search.matches() {
        if m.line != line_idx || m.end_col <= start_col {
            continue;
        }

        let match_start = char_to_byte_idx(line, m.start_col).saturating_sub(start_byte);
        let match_end = char_to_byte_idx(line, m.end_col).saturating_sub(start_byte);
        let text_before_len = match_start.saturating_sub(current_pos);

        if text_before_len > 0 && current_pos < visible_line.len() {
            let end_idx = (current_pos + text_before_len).min(visible_line.len());
            highlighted_line.push_str(&visible_line[current_pos..end_idx]);
            current_pos = end_idx;
        }

        let match_start = match_start.min(visible_line.len());
        let match_end = match_end.min(visible_line.len());

        if match_start < visible_line.len() && match_end > match_start {
            write!(
                highlighted_line,
                "{}{}{}",
                SetBackgroundColor(Color::Yellow),
                &visible_line[match_start..match_end],
                line_bg
                    .map(SetBackgroundColor)
                    .unwrap_or(ResetColor)
            )
            .unwrap();
            current_pos = match_end;
        }
    }

    if current_pos < visible_line.len() {
        highlighted_line.push_str(&visible_line[current_pos..]);
    }

    if line_bg.is_some() {
        write!(highlighted_line, "{}", ResetColor).unwrap();
    }

    highlighted_line
}

fn char_to_byte_idx(text: &str, char_idx: usize) -> usize {
    if char_idx == 0 {
        return 0;
    }

    text.char_indices()
        .nth(char_idx)
        .map(|(idx, _)| idx)
        .unwrap_or_else(|| text.len())
}

pub fn render_status_bar<W: Write>(
    stdout: &mut W,
    row: u16,
    filename: Option<&str>,
    cursor_line: usize,
    total_lines: usize,
    cursor_col: usize,
) {
    let file_info = filename.unwrap_or("[Sin nombre]");
    let width = terminal::size().map(|(width, _)| width as usize).unwrap_or(0);
    let status_text = format!(
        "{} | Linea {}/{}, Col {}",
        file_info, cursor_line, total_lines, cursor_col
    );
    let visible_text = truncate_with_ellipsis(&status_text, width);
    let padded_text = pad_to_width(&visible_text, width);
    write!(
        stdout,
        "{}{}",
        cursor::MoveTo(0, row),
        SetBackgroundColor(Color::White)
    )
    .unwrap();
    write!(stdout, "{}", SetForegroundColor(Color::Black)).unwrap();
    write!(stdout, "{}", padded_text).unwrap();
    write!(stdout, "{}", ResetColor).unwrap();
    write!(
        stdout,
        "{}",
        terminal::Clear(terminal::ClearType::UntilNewLine)
    )
    .unwrap();
}

pub fn render_message<W: Write>(stdout: &mut W, row: u16, message: &str) {
    let width = terminal::size().map(|(width, _)| width as usize).unwrap_or(0);
    let visible_message = truncate_with_ellipsis(message, width);
    let padded_message = pad_to_width(&visible_message, width);
    write!(
        stdout,
        "{}{}{}",
        cursor::MoveTo(0, row),
        padded_message,
        terminal::Clear(terminal::ClearType::UntilNewLine)
    )
    .unwrap();
}

pub fn calculate_line_number_width(total_lines: usize) -> usize {
    total_lines.to_string().len() + 2
}

pub fn calculate_visual_cursor_position(
    cursor_x: usize,
    cursor_y: usize,
    offset_col: usize,
    offset_row: usize,
    line_num_width: usize,
) -> (u16, u16) {
    let visual_x = cursor_x
        .saturating_sub(offset_col)
        .saturating_add(line_num_width);
    let visual_y = cursor_y.saturating_sub(offset_row);
    (visual_x as u16, visual_y as u16)
}

pub fn position_cursor<W: Write>(stdout: &mut W, x: u16, y: u16) {
    write!(stdout, "{}", cursor::MoveTo(x, y)).unwrap();
}

fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    let text_width = text.chars().count();
    if max_width == 0 {
        return String::new();
    }
    if text_width <= max_width {
        return text.to_string();
    }
    if max_width == 1 {
        return "…".to_string();
    }
    let truncated: String = text.chars().take(max_width - 1).collect();
    format!("{}…", truncated)
}

fn pad_to_width(text: &str, width: usize) -> String {
    let text_width = text.chars().count();
    if text_width >= width {
        return text.to_string();
    }
    let mut padded = String::with_capacity(width);
    padded.push_str(text);
    padded.extend(std::iter::repeat(' ').take(width - text_width));
    padded
}

/**pub fn clear_screen<W: Write>(stdout: &mut W) {
    write!(stdout, "{}", terminal::Clear(terminal::ClearType::All)).unwrap();
}**/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_line_number_width() {
        assert_eq!(calculate_line_number_width(9), 3);
        assert_eq!(calculate_line_number_width(99), 4);
        assert_eq!(calculate_line_number_width(999), 5);
    }

    #[test]
    fn test_calculate_visual_cursor_position() {
        let (x, y) = calculate_visual_cursor_position(10, 5, 0, 0, 4);
        assert_eq!(x, 14);
        assert_eq!(y, 5);

        let (x, y) = calculate_visual_cursor_position(50, 5, 30, 0, 4);
        assert_eq!(x, 24);
        assert_eq!(y, 5);

        let (x, y) = calculate_visual_cursor_position(10, 25, 0, 10, 4);
        assert_eq!(x, 14);
        assert_eq!(y, 15);
    }
}
