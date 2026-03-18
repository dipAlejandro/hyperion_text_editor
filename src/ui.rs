use crate::config::SyntaxTheme;
use crate::search::SearchState;
use crate::syntax::{SyntaxLanguage, detect_language, tokenize_line};
use crossterm::{
    cursor,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::fmt::Write as _;
use std::io::Write;

#[derive(Clone, Copy)]
pub struct SyntaxRenderConfig<'a> {
    pub language: SyntaxLanguage,
    pub syntax_theme: &'a SyntaxTheme,
}

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
    language: SyntaxLanguage,
) {
    let line_bg = is_current_line.then_some(Color::DarkGrey);
    let chars: Vec<char> = line.chars().collect();
    let tokens = tokenize_line(line, language);
    let mut styled = String::new();
    let mut prev_style: Option<(Option<Color>, Option<Color>)> = None;

    for (col, ch) in chars.iter().enumerate().skip(start_col) {
        let fg = tokens.get(col).and_then(|token| token.map(|t| t.color()));
        let bg = if is_match_col(line_idx, search, col) {
            Some(Color::Yellow)
        } else {
            line_bg
        };
        let style = Some((fg, bg));

        if prev_style != style {
            write!(styled, "{}", ResetColor).unwrap();
            if let Some(bg) = bg {
                write!(styled, "{}", SetBackgroundColor(bg)).unwrap();
            }
            if let Some(fg) = fg {
                write!(styled, "{}", SetForegroundColor(fg)).unwrap();
            }
            prev_style = style;
        }

        styled.push(*ch);
    }

    if !chars.is_empty() && prev_style.is_some() {
        write!(styled, "{}", ResetColor).unwrap();
    }

    write!(stdout, "{}", styled).unwrap();
}

fn is_match_col(line_idx: usize, search: &SearchState, col: usize) -> bool {
    if !search.is_active() {
        return false;
    }

    search
        .matches()
        .iter()
        .any(|m| m.line == line_idx && col >= m.start_col && col < m.end_col)
}

pub fn language_from_filename(filename: Option<&str>) -> SyntaxLanguage {
    detect_language(filename)
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
    let width = terminal::size()
        .map(|(width, _)| width as usize)
        .unwrap_or(0);
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
    let width = terminal::size()
        .map(|(width, _)| width as usize)
        .unwrap_or(0);
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
    padded.extend(std::iter::repeat_n(' ', width - text_width));
    padded
}

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
