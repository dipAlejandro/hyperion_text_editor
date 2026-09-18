use crate::config::LanguageConfig;
use crate::config::SyntaxTheme;
use crate::search::SearchState;
use crate::syntax::{detect_language, tokenize_line, SyntaxLanguage};
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

#[derive(Clone, Copy)]
pub struct LineViewport {
    pub start_col: usize,
    pub visible_cols: usize,
}

pub fn render_line_number<W: Write>(stdout: &mut W, line_number: usize, row: u16, width: usize) {
    write!(stdout, "{}", cursor::MoveTo(0, row)).unwrap();
    write!(stdout, "{}", SetForegroundColor(Color::Cyan)).unwrap();
    write!(stdout, "{:>width$} ", line_number, width = width - 1).unwrap();
    write!(stdout, "{}", ResetColor).unwrap();
}
pub fn render_tab_bar<W: Write>(
    stdout: &mut W,
    width: usize,
    tabs: &[(String, bool, bool)],
    bg: Color,
    fg: Color,
) {
    let mut text = String::new();
    for (label, is_active, dirty) in tabs {
        let marker = if *dirty { "*" } else { "" };
        if *is_active {
            text.push_str(&format!("[{}{}] ", label, marker));
        } else {
            text.push_str(&format!(" {}{}  ", label, marker));
        }
    }

    let visible = truncate_with_ellipsis(&text, width);
    let padded = pad_to_width(&visible, width);

    write!(stdout, "{}", cursor::MoveTo(0, 0)).unwrap();
    write!(stdout, "{}", SetBackgroundColor(bg)).unwrap();
    write!(stdout, "{}", SetForegroundColor(fg)).unwrap();
    write!(stdout, "{}", padded).unwrap();
    write!(stdout, "{}", ResetColor).unwrap();
}
pub fn render_line_content<W: Write>(
    stdout: &mut W,
    line: &str,
    line_idx: usize,
    viewport: LineViewport,
    search: &SearchState,
    syntax: SyntaxRenderConfig<'_>,
    selection: Option<((usize, usize), (usize, usize))>,
    selection_bg: Color,
) {
    let chars: Vec<char> = line.chars().collect();
    let tokens = tokenize_line(line, syntax.language);
    let mut styled = String::new();
    let mut prev_style: Option<(Option<Color>, Option<Color>)> = None;

    for (col, ch) in chars
        .iter()
        .enumerate()
        .skip(viewport.start_col)
        .take(viewport.visible_cols)
    {
        let fg = tokens
            .get(col)
            .and_then(|token| token.map(|t| color_for_token(t, syntax.syntax_theme)));

        let bg = if is_selected_col(line_idx, col, selection) {
            Some(selection_bg)
        } else if is_match_col(line_idx, search, col) {
            Some(Color::Yellow)
        } else {
            None
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

fn is_selected_col(
    line_idx: usize,
    col: usize,
    selection: Option<((usize, usize), (usize, usize))>,
) -> bool {
    let Some((start, end)) = selection else {
        return false;
    };

    if line_idx < start.0 || line_idx > end.0 {
        return false;
    }

    let from = if line_idx == start.0 { start.1 } else { 0 };
    let to = if line_idx == end.0 { end.1 } else { usize::MAX };

    col >= from && col < to
}

fn color_for_token(token: crate::syntax::TokenKind, theme: &SyntaxTheme) -> Color {
    match token {
        crate::syntax::TokenKind::Keyword => theme.keyword,
        crate::syntax::TokenKind::String => theme.string,
        crate::syntax::TokenKind::Number => theme.number,
        crate::syntax::TokenKind::Comment => theme.comment,
    }
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

pub fn language_from_filename(
    filename: Option<&str>,
    languages: &LanguageConfig,
) -> SyntaxLanguage {
    detect_language(filename, languages)
}
pub fn render_status_bar<W: Write>(
    stdout: &mut W,
    row: u16,
    width: usize,
    filename: Option<&str>,
    cursor_line: usize,
    total_lines: usize,
    cursor_col: usize,
    dirty: bool,
    bg: Color,
    fg: Color,
) {
    let file_info = filename.unwrap_or("[Sin nombre]");
    let dirty_marker = if dirty { " *" } else { "" };
    let status_text = format!(
        "{}{} | Linea {}/{}, Col {}",
        file_info, dirty_marker, cursor_line, total_lines, cursor_col
    );
    let visible_text = truncate_with_ellipsis(&status_text, width);
    let padded_text = pad_to_width(&visible_text, width);
    write!(
        stdout,
        "{}{}",
        cursor::MoveTo(0, row),
        SetBackgroundColor(bg)
    )
    .unwrap();
    write!(stdout, "{}", SetForegroundColor(fg)).unwrap();
    write!(stdout, "{}", padded_text).unwrap();
    write!(stdout, "{}", ResetColor).unwrap();
    write!(
        stdout,
        "{}",
        terminal::Clear(terminal::ClearType::UntilNewLine)
    )
    .unwrap();
}

pub fn render_message<W: Write>(stdout: &mut W, row: u16, width: usize, message: &str) {
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
pub struct PickerViewport<'a> {
    pub query: &'a str,
    pub entries: &'a [crate::picker::PickerEntry],
    pub selected: usize,
    pub scroll_offset: usize,
}

pub fn render_picker<W: Write>(stdout: &mut W, viewport: PickerViewport<'_>) {
    let width = terminal::size()
        .map(|(width, _)| width as usize)
        .unwrap_or(0);
    let (_, height) = terminal::size().unwrap_or((80, 24));
    let visible_rows = (height as usize).saturating_sub(1);

    write!(stdout, "{}", cursor::MoveTo(0, 0)).unwrap();
    write!(
        stdout,
        "{}",
        terminal::Clear(terminal::ClearType::FromCursorDown)
    )
    .unwrap();

    let prompt = format!("Buscar: {}", viewport.query);
    let visible_prompt = truncate_with_ellipsis(&prompt, width);
    write!(stdout, "{}", pad_to_width(&visible_prompt, width)).unwrap();

    let start = viewport.scroll_offset;
    let end = (start + visible_rows).min(viewport.entries.len());

    for (row, entry) in viewport.entries[start..end].iter().enumerate() {
        let line_row = (row + 1) as u16;
        let is_selected = start + row == viewport.selected;
        let is_dir = matches!(entry.kind, crate::picker::PickerEntryKind::Dir);

        write!(stdout, "{}", cursor::MoveTo(0, line_row)).unwrap();

        if is_selected {
            write!(stdout, "{}", SetBackgroundColor(Color::DarkGrey)).unwrap();
        }

        // Sufijo "/" para directorios, igual que netrw/ls -p
        let display_name = if is_dir {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };

        let visible_name = truncate_with_ellipsis(&display_name, width);
        let visible_len = visible_name.chars().count();

        let base_fg = if is_dir { Color::Cyan } else { Color::White };

        let mut styled = String::new();
        write!(styled, "{}", SetForegroundColor(base_fg)).unwrap();
        for (col, ch) in visible_name.chars().enumerate() {
            if entry.match_indices.contains(&col) {
                write!(styled, "{}", SetForegroundColor(Color::Yellow)).unwrap();
                styled.push(ch);
                write!(styled, "{}", SetForegroundColor(base_fg)).unwrap();
                if is_selected {
                    write!(styled, "{}", SetBackgroundColor(Color::DarkGrey)).unwrap();
                }
            } else {
                styled.push(ch);
            }
        }
        write!(styled, "{}", ResetColor).unwrap();

        write!(stdout, "{}", styled).unwrap();

        if is_selected {
            write!(stdout, "{}", SetBackgroundColor(Color::DarkGrey)).unwrap();
        }
        if visible_len < width {
            write!(stdout, "{}", " ".repeat(width - visible_len)).unwrap();
        }

        write!(stdout, "{}", ResetColor).unwrap();
    }

    write!(
        stdout,
        "{}",
        cursor::MoveTo((8 + viewport.query.chars().count()) as u16, 0)
    )
    .unwrap();
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
