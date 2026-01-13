use crate::search::SearchState;
use std::fmt::Write as _;
use std::io::Write;

/// Renderiza los numeros de linea en color cyan
///
/// #Args
/// * `stdout` - Terminal donde escribir
/// * `line_number` - Numero de linea a mostrar (base 1)
/// * `row` - Fila de la pantalla donde dibujar
/// * `width` - Ancho para alinear los numeros
pub fn render_line_number<W: Write>(stdout: &mut W, line_number: usize, row: u16, width: usize) {
    write!(
        stdout,
        "{}{}{:>width$} {}",
        termion::cursor::Goto(1, row),
        termion::color::Fg(termion::color::Cyan),
        line_number,
        termion::color::Fg(termion::color::Reset),
        width = width
    )
    .unwrap();
}

/// Renderiza una línea de texto con resaltado de búsqueda opcional
///
/// # Argumentos
/// * `stdout` - Terminal donde escribir
/// * `line` - Texto de la línea a renderizar
/// * `line_idx` - Índice de la línea en el documento (base 0)
/// * `start_col` - Columna inicial visible (para scroll horizontal)
/// * `search` - Estado de búsqueda (opcional)
pub fn render_line_content<W: Write>(
    stdout: &mut W,
    line: &str,
    line_idx: usize,
    start_col: usize,
    search: &SearchState,
) {
    let visible_line = &line[start_col.min(line.len())..];

    if search.is_active() {
        let highlighted = highlight_matches(visible_line, line_idx, start_col, search);
        write!(stdout, "{}", highlighted).unwrap();
    } else {
        write!(stdout, "{}", visible_line).unwrap();
    }
}

/// Aplica resaltado a las coincidencias de búsqueda en una línea
///
/// # Argumentos
/// * `visible_line` - Porción visible de la línea
/// * `line_idx` - Índice de la línea en el documento
/// * `start_col` - Columna de inicio (offset horizontal)
/// * `search` - Estado de búsqueda
///
/// # Retorna
/// String con las coincidencias resaltadas usando códigos de color
fn highlight_matches(
    visible_line: &str,
    line_idx: usize,
    start_col: usize,
    search: &SearchState,
) -> String {
    let mut highlighted_line = String::new();
    let mut current_pos = 0;

    // Iterar sobre todas las coincidencias de búsqueda
    for m in search.matches() {
        // Solo procesar coincidencias en esta línea y que sean visibles
        if m.line != line_idx || m.start_col < start_col {
            continue;
        }

        // Añadir texto normal antes de la coincidencia
        let text_before_len = m
            .start_col
            .saturating_sub(start_col)
            .saturating_sub(current_pos);

        if text_before_len > 0 && current_pos < visible_line.len() {
            let end_idx = (current_pos + text_before_len).min(visible_line.len());
            highlighted_line.push_str(&visible_line[current_pos..end_idx]);
            current_pos = end_idx;
        }

        // Añadir coincidencia resaltada con fondo amarillo
        let match_start = m.start_col.saturating_sub(start_col);
        let match_end = m.end_col.saturating_sub(start_col).min(visible_line.len());

        if match_start < visible_line.len() && match_end > match_start {
            write!(
                highlighted_line,
                "{}{}{}",
                termion::color::Bg(termion::color::Yellow),
                &visible_line[match_start..match_end],
                termion::color::Bg(termion::color::Reset)
            )
            .unwrap();
            current_pos = match_end;
        }
    }

    // Añadir el resto de la línea después de todas las coincidencias
    if current_pos < visible_line.len() {
        highlighted_line.push_str(&visible_line[current_pos..]);
    }

    highlighted_line
}

/// Renderiza la barra de estado en la parte inferior de la pantalla
///
/// # Argumentos
/// * `stdout` - Terminal donde escribir
/// * `row` - Fila donde dibujar la barra
/// * `filename` - Nombre del archivo (o None si no tiene nombre)
/// * `cursor_line` - Línea actual del cursor (base 1)
/// * `total_lines` - Total de líneas en el documento
/// * `cursor_col` - Columna actual del cursor (base 1)
pub fn render_status_bar<W: Write>(
    stdout: &mut W,
    row: u16,
    filename: Option<&str>,
    cursor_line: usize,
    total_lines: usize,
    cursor_col: usize,
) {
    let file_info = filename.unwrap_or("[Sin nombre]");
    write!(
        stdout,
        "{}{}{} | Linea {}/{}, Col {}{}",
        termion::cursor::Goto(1, row),
        termion::style::Invert,
        file_info,
        cursor_line,
        total_lines,
        cursor_col,
        termion::style::Reset
    )
    .unwrap();
    // Limpiar cualquier texto residual
    write!(stdout, "{}", termion::clear::AfterCursor).unwrap();
}

/// Renderiza el mensaje de estado en la última línea
///
/// # Argumentos
/// * `stdout` - Terminal donde escribir
/// * `row` - Fila donde dibujar el mensaje
/// * `message` - Mensaje a mostrar
pub fn render_message<W: Write>(stdout: &mut W, row: u16, message: &str) {
    write!(stdout, "{}{}", termion::cursor::Goto(1, row), message).unwrap();
}

/// Calcula el ancho necesario para mostrar los números de línea
///
/// # Argumentos
/// * `total_lines` - Número total de líneas en el documento
///
/// # Retorna
/// El ancho en caracteres (incluyendo un espacio de separación)
pub fn calculate_line_number_width(total_lines: usize) -> usize {
    total_lines.to_string().len() + 2
}

/// Calcula la posición visual del cursor en la pantalla
///
/// # Argumentos
/// * `cursor_x` - Posición X del cursor en el documento
/// * `cursor_y` - Posición Y del cursor en el documento
/// * `offset_col` - Offset de scroll horizontal
/// * `offset_row` - Offset de scroll vertical
/// * `line_num_width` - Ancho de la columna de números de línea
///
/// # Retorna
/// Tupla (x, y) con la posición visual en la pantalla
pub fn calculate_visual_cursor_position(
    cursor_x: usize,
    cursor_y: usize,
    offset_col: usize,
    offset_row: usize,
    line_num_width: usize,
) -> (u16, u16) {
    let visual_x = (cursor_x.saturating_sub(offset_col) + line_num_width) as u16;
    let visual_y = (cursor_y.saturating_sub(offset_row) + 1) as u16;
    (visual_x, visual_y)
}

/// Posiciona el cursor en una ubicación específica de la pantalla
///
/// # Argumentos
/// * `stdout` - Terminal donde escribir
/// * `x` - Coordenada X (columna)
/// * `y` - Coordenada Y (fila)
pub fn position_cursor<W: Write>(stdout: &mut W, x: u16, y: u16) {
    write!(stdout, "{}", termion::cursor::Goto(x, y)).unwrap();
}

/// Limpia toda la pantalla
pub fn clear_screen<W: Write>(stdout: &mut W) {
    write!(stdout, "{}", termion::clear::All).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_line_number_width() {
        assert_eq!(calculate_line_number_width(9), 3); // 1 dígito + 2
        assert_eq!(calculate_line_number_width(99), 4); // 2 dígitos + 2
        assert_eq!(calculate_line_number_width(999), 5); // 3 dígitos + 2
    }

    #[test]
    fn test_calculate_visual_cursor_position() {
        // Sin scroll, con números de línea de ancho 4
        let (x, y) = calculate_visual_cursor_position(10, 5, 0, 0, 4);
        assert_eq!(x, 14); // 10 + 4
        assert_eq!(y, 6); // 5 + 1

        // Con scroll horizontal
        let (x, y) = calculate_visual_cursor_position(50, 5, 30, 0, 4);
        assert_eq!(x, 24); // (50 - 30) + 4
        assert_eq!(y, 6);

        // Con scroll vertical
        let (x, y) = calculate_visual_cursor_position(10, 25, 0, 10, 4);
        assert_eq!(x, 14); // 10 + 4
        assert_eq!(y, 16); // (25 - 10) + 1
    }
}
