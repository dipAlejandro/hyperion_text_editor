mod search;
mod terminal;

use std::{
    fs,
    io::{self, Write},
};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

use crate::terminal::request_input;

// Estructura que representa coincidencia de busqueda
#[derive(Clone, Debug)]
struct Match {
    line: usize,      // En que linea está la coincidencia
    start_col: usize, // En que columna empieza
    end_col: usize,   // En que columna termina
}

struct Editor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    filename: Option<String>,
    state_msg: String,
    window_sizes: (u16, u16),
    offset_row: usize,
    offset_col: usize,
    search_query: Option<String>, // Texto que estamos buscando actualmente
    search_matches: Vec<Match>,   // Todas las coincidencias encontradas
    current_match_index: Option<usize>, // En cual coincidencia estamos posicionados
}

impl Editor {
    fn new() -> Self {
        let window_sizes = termion::terminal_size().unwrap_or((80, 24));

        Editor {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            filename: None,
            state_msg: String::from("Ctrl+Q: Salir | Ctrl+S: Guardar | Ctrl+O: Abrir"),
            window_sizes,
            offset_row: 0,
            offset_col: 0,
            search_query: None,
            search_matches: Vec::new(),
            current_match_index: None,
        }
    }

    fn open_file(&mut self, path: &str) {
        // Intentar leer el archivo completo como un string
        match fs::read_to_string(path) {
            Ok(content) => {
                // Si se logra, dividir el contenido en lineas
                // El collect() convierte el iterador en un Vec<String>
                self.lines = content.lines().map(|line| line.to_string()).collect();

                // Si el archivo esta vacio asegurar tener al menos una linea
                if self.lines.is_empty() {
                    self.lines.push(String::new());
                }

                // Guardar el nombre del archivo y resetear el cursor
                self.filename = Some(path.to_string());
                self.cursor_x = 0;
                self.cursor_y = 0;
                self.offset_row = 0;
                self.offset_col = 0;
                self.state_msg = format!("Archivo '{}' cargado correctamente", path);
            }
            Err(e) => {
                self.state_msg = format!("Error al abrir el archivo: {}", e);
            }
        }
    }

    fn save_file(&mut self, path: &str) {
        let content = self.lines.join("\n");

        //Intentar escribir el contenido del archivo
        match fs::write(path, content) {
            Ok(_) => {
                self.filename = Some(path.to_string());
                self.state_msg = format!("Archivo '{}' guardado correctamente.", path);
            }
            Err(e) => {
                self.state_msg = format!("Error al intentar guardar el archivo: {}", e);
            }
        }
    }

    fn insert_char(&mut self, c: char) {
        let current_line = &mut self.lines[self.cursor_y];
        current_line.insert(self.cursor_x, c);

        self.cursor_x += 1;
    }

    fn new_line(&mut self) {
        let current_line = &self.lines[self.cursor_y];
        let right_txt = current_line[self.cursor_x..].to_string();

        self.lines[self.cursor_y].truncate(self.cursor_x);

        self.lines.insert(self.cursor_y + 1, right_txt);

        self.cursor_x = 0;
        self.cursor_y += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_x > 0 {
            // Si no estamos al inicio de la línea, borramos el carácter anterior
            self.lines[self.cursor_y].remove(self.cursor_x - 1);
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            // Si estamos al inicio de la línea pero no en la primera línea
            // juntamos esta línea con la anterior
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
            self.lines[self.cursor_y].push_str(&current_line);
        }
    }

    fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            // Ajustamos cursor_x si la nueva línea es más corta
            let line_length = self.lines[self.cursor_y].len();
            if self.cursor_x > line_length {
                self.cursor_x = line_length;
            }
        }
    }

    fn move_down(&mut self) {
        if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;

            // Ajustamos cursor_x si la nueva línea es más corta
            let line_length = self.lines[self.cursor_y].len();
            if self.cursor_x > line_length {
                self.cursor_x = line_length;
            }
        }
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            // Si estamos al inicio de una línea, vamos al final de la línea anterior
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
        }
    }

    fn move_right(&mut self) {
        let line_length = self.lines[self.cursor_y].len();
        if self.cursor_x < line_length {
            self.cursor_x += 1;
        } else if self.cursor_y < self.lines.len() - 1 {
            // Si estamos al final de una línea, vamos al inicio de la siguiente
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    fn adjust_scroll(&mut self) {
        // Calcular cuantas lineas se pueden mostrar (dejar 2 para barra de estado)
        let visible_lines = (self.window_sizes.1 - 2) as usize;

        // Si el cursor está por encima de la ventana visible, ajustamos hacia arriba
        if self.cursor_y < self.offset_row {
            self.offset_row = self.cursor_y;
        }

        // Si el cursor está por debajo de la ventana visible, ajustar hacia abajo
        // Restar 1 porque las posiciones empiezan en 0
        if self.cursor_y >= self.offset_row + visible_lines {
            self.offset_row = self.cursor_y - visible_lines + 1;
        }

        // Calcular el ancho del area de numeros de linea
        // Necesitamos saber cuantos digitos tiene el numero de linea mas alto
        let line_num_digits = self.lines.len().to_string().len();
        let line_num_width = line_num_digits + 2;

        // Calcular cuantas columnas tenemos disponibles para el texto
        let visible_cols = (self.window_sizes.0 as usize).saturating_sub(line_num_width);

        // Ajustar scroll horizontal si es necesario
        if self.cursor_x < self.offset_col {
            self.offset_col = self.cursor_x;
        }

        if self.cursor_x >= self.offset_col + visible_cols {
            self.offset_col = self.cursor_x - visible_cols + 1;
        }
    }

    fn search(&mut self, query: &str) {
        // Si la busqueda está vacia, limpiar y salir
        if query.is_empty() {
            self.search_query = None;
            self.search_matches.clear();
            self.current_match_index = None;
            self.state_msg = String::from("Busqueda cancelada");
            return;
        }

        // Guardar consulta y limpiar coincidencias
        self.search_query = Some(query.to_string());
        self.search_matches.clear();

        for (line_idx, line) in self.lines.iter().enumerate() {
            let mut start_pos = 0;

            // Este bucle continúa buscando hasta que no encuentre más coincidencias
            while let Some(found_pos) = line[start_pos..].find(query) {
                // Calcular la posicion en la linea completa
                let current_pos = start_pos + found_pos;

                self.search_matches.push(Match {
                    line: line_idx,
                    start_col: current_pos,
                    end_col: current_pos + query.len(),
                });

                start_pos = current_pos + 1;
            }
        }

        // Si se encuentra coincidencias, mover a la primera
        if !self.search_matches.is_empty() {
            self.current_match_index = Some(0);
            self.jump_to_current_match();
            self.state_msg = format!(
                "Encontradas {} coincidencias de '{}'",
                self.search_matches.len(),
                query
            );
        } else {
            self.current_match_index = None;
            self.state_msg = format!("No se encontró '{}'", query);
        }
    }

    fn jump_to_current_match(&mut self) {
        if let Some(idx) = self.current_match_index {
            if let Some(m) = self.search_matches.get(idx) {
                // Mover el cursor al inicio de la coincidencia
                self.cursor_y = m.line;
                self.cursor_x = m.start_col;

                self.state_msg = format!(
                    "Coincidencias {}/{}: '{}'",
                    idx + 1,
                    self.search_matches.len(),
                    self.search_query.as_ref().unwrap_or(&String::new())
                );
            }
        }
    }

    fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            self.state_msg = String::from("No hay busqueda activa");
            return;
        }

        if let Some(current_idx) = self.current_match_index {
            self.current_match_index = Some((current_idx + 1) % self.search_matches.len());
            self.jump_to_current_match();
        }
    }

    fn previous_match(&mut self) {
        if self.search_matches.is_empty() {
            self.state_msg = String::from("No hay busqueda activa");
            return;
        }

        if let Some(current_idx) = self.current_match_index {
            let total_matches = self.search_matches.len();
            self.current_match_index = Some((current_idx + total_matches - 1) % total_matches);
            self.jump_to_current_match();
        }
    }

    /**
     * coord: (y, x)
     * */
    fn go_to_line(&mut self, coords: (usize, usize)) {
        // Validar que la línea existe
        // Recordar que coords.0 es base-1 (como lo ve el usuario)
        // pero internamente trabajamos con base-0
        if coords.0 >= self.lines.len() {
            // La línea no existe, mostrar error y no mover el cursor
            self.state_msg = format!(
                "Línea {} no existe. El documento tiene {} líneas",
                coords.0 + 1,
                self.lines.len()
            );
            return;
        }

        // La línea es válida, mover el cursor
        self.cursor_y = coords.0;

        // Ahora validar la columna basándonos en la línea donde acabamos de posicionar el cursor
        let line_length = self.lines[self.cursor_y].len();

        if coords.1 >= line_length {
            // Si la columna está fuera de rango, ir al final de la línea
            self.cursor_x = line_length;
            self.state_msg = format!(
                "Columna {} fuera de rango. Posicionado al final de la línea (columna {})",
                coords.1 + 1,
                line_length
            );
        } else {
            // La columna es válida
            self.cursor_x = coords.1;
            self.state_msg = format!(
                "Posicionado en línea {}, columna {}",
                self.cursor_y + 1,
                self.cursor_x + 1
            );
        }
    }

    fn write<W: Write>(&self, stdout: &mut W) {
        write!(stdout, "{}", termion::clear::All).unwrap();

        // Calcular cuantas lineas se pueden mostrar (dejar 2 para barra de estado)
        let visible_lines = (self.window_sizes.1 - 2) as usize;

        // Calcular ancho necesario para los numero de linea
        // Esto depende de cuantas lineas tiene el documento en total
        let line_num_digits = self.lines.len().to_string().len();
        let line_num_width = line_num_digits + 2;

        // Iterar solo sobre las lineas que estan actualmente visibles
        // self.offset_row nos dice cual es la primera linea visible
        let start = self.offset_row;
        let end = (self.offset_row + visible_lines).min(self.lines.len());

        for i in start..end {
            let line_num = i + 1;
            let window_row = (i - self.offset_row + 1) as u16;

            // Dibujar el número de línea con un color diferente
            // El formato {:>width$} alinea el número a la derecha
            write!(
                stdout,
                "{}{}{}{}",
                termion::cursor::Goto(1, window_row),
                termion::color::Fg(termion::color::Cyan),
                format!("{:>width$} ", line_num, width = line_num_digits),
                termion::color::Fg(termion::color::Reset)
            )
            .unwrap();

            // Obtenemos la porcion visible  de la linea considerando el scroll horizontal
            let line = &self.lines[i];
            let start_col = self.offset_col.min(line.len());
            let visible_line = &line[start_col..];

            // Si hay una busqueda activa resaltar las coincidencias
            if let Some(query) = &self.search_query {
                let mut current_pos = 0;
                let mut highlighted_line = String::new();

                // Buscar todas las coincidencias en esta linea que esten visibles
                for m in &self.search_matches {
                    if m.line == i && m.start_col >= start_col {
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

                        // Añadir coincidencia resaltada
                        let match_start = m.start_col.saturating_sub(start_col);
                        let match_end = m.end_col.saturating_sub(start_col).min(visible_line.len());

                        if match_start < visible_line.len() && match_end > match_start {
                            highlighted_line.push_str(&format!(
                                "{}{}{}",
                                termion::color::Bg(termion::color::Yellow),
                                &visible_line[match_start..match_end],
                                termion::color::Bg(termion::color::Reset)
                            ));
                            current_pos = match_end;
                        }
                    }
                }
                // Añadir el resto de la linea después de todas las coincidencia
                if current_pos < visible_line.len() {
                    highlighted_line.push_str(&visible_line[current_pos..]);
                }

                write!(stdout, "{}", highlighted_line).unwrap();
            } else {
                write!(stdout, "{}", visible_line).unwrap();
            }
        }

        // Dibujar la barra de estado en la parte inferior
        let state_row = self.window_sizes.1 - 1;

        // Crear la información de la barra de estado
        let info_file = match &self.filename {
            Some(name) => name.clone(),
            None => String::from("[Sin nombre]"),
        };

        let info_position = format!(
            "Linea {}/{}, Col {}",
            self.cursor_y + 1,
            self.lines.len(),
            self.cursor_x + 1
        );

        // Dibujar la barra de estado con fondo invertido
        write!(
            stdout,
            "{}{}{}{}",
            termion::cursor::Goto(1, state_row),
            termion::style::Invert,
            format!("{} | {}", info_file, info_position),
            termion::style::Reset
        )
        .unwrap();

        // Limpiar cualquier texto que pudiera quedar de la linea de estado anterior
        write!(stdout, "{}", termion::clear::AfterCursor).unwrap();

        // Dibujar la barra de estado en la ultima linea
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, state_row + 1),
            &self.state_msg
        )
        .unwrap();

        // Calcular la posicion visual del cursor en la pantalla
        // Tenemos que considerar el offset de scroll y el ancho de los numeros de linea
        let visual_cursor_x =
            (self.cursor_x.saturating_sub(self.offset_col) + line_num_width) as u16;
        let visual_cursor_y = (self.cursor_y.saturating_sub(self.offset_row) + 1) as u16;

        write!(
            stdout,
            "{}",
            termion::cursor::Goto(visual_cursor_x, visual_cursor_y)
        )
        .unwrap();

        stdout.flush().unwrap();
    }
}

fn main() {
    // Activar modo raw en la terminal
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let stdin = io::stdin();

    let mut editor = Editor::new();

    editor.write(&mut stdout);

    write!(
        stdout,
        "{}{}Editor de Texto - Presiona 'q' para salir \r\n\r\n",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();

    stdout.flush().unwrap();

    // Leer entrada de usuario
    for k in stdin.keys() {
        // Limpiar el mensaje de estado antes de procesar la siguiente tecla
        // Esto hace que los mensajes temporales desaparezcan después de cualquier acción

        if !editor.state_msg.starts_with("Ctrl+") {
            editor.state_msg = String::from("Ctrl+Q: Salir | Ctrl+S: Guardar | Ctrl+O: Abrir");
        }
        match k.unwrap() {
            //Ctrl + Q para salir
            Key::Ctrl('q') => break,

            // Ctrl+S para guardar
            Key::Ctrl('s') => {
                let path = match &editor.filename {
                    Some(name) => name.clone(),
                    None => {
                        // Si no hay nombre de archivo pedimos uno
                        let name = request_input(&mut stdout, "Guardar como: ");
                        if name.is_empty() {
                            editor.state_msg = String::from("Guardado cancelado");
                            editor.write(&mut stdout);
                            continue;
                        }
                        name
                    }
                };
                editor.save_file(&path);
            }

            // Ctrl+O para abrir archivo
            Key::Ctrl('o') => {
                let path = request_input(&mut stdout, "Abrir archivo: ");
                if !path.is_empty() {
                    editor.open_file(&path);
                } else {
                    editor.state_msg = String::from("Apertura cancelada");
                }
            }
            // Ctrl + F para buscar
            Key::Ctrl('f') => {
                let query = request_input(&mut stdout, "Buscar: ");
                editor.search(&query);
            }

            //  Ctrl + N para siguiente coincidencia
            Key::Ctrl('n') => editor.next_match(),

            // Ctrl + P para coincidencia anterior
            Key::Ctrl('p') => editor.previous_match(),

            // Ctrl + G
            Key::Ctrl('g') => {
                let coords_str = request_input(&mut stdout, "Ir a (linea, columna): ");

                // Intentar parsear las coordenadas de manera segura
                let parts: Vec<&str> = coords_str.split(',').collect();

                // Verificar que tenemos exactamente dos partes
                if parts.len() != 2 {
                    editor.state_msg = String::from("Formato inválido. Use: linea,columna");
                    editor.write(&mut stdout);
                    continue;
                }

                // Intentar parsear cada parte como número
                match (
                    parts[0].trim().parse::<usize>(),
                    parts[1].trim().parse::<usize>(),
                ) {
                    (Ok(line), Ok(col)) => {
                        // Verificar que los números no sean cero (el usuario ingresa base-1)
                        if line == 0 || col == 0 {
                            editor.state_msg = String::from("Las líneas y columnas empiezan en 1");
                        } else {
                            // Convertir de base-1 (usuario) a base-0 (interno)
                            editor.go_to_line((line - 1, col - 1));
                        }
                    }
                    _ => {
                        editor.state_msg = String::from("Ingrese números válidos");
                    }
                }
            }

            //Teclas de navegación
            Key::Up => editor.move_up(),
            Key::Down => editor.move_down(),
            Key::Left => editor.move_left(),
            Key::Right => editor.move_right(),

            // Enter crea nueva linea
            Key::Char('\n') => editor.new_line(),

            // Backspace para borrar caracteres
            Key::Backspace => editor.delete_char(),

            // Cualquier otro caracter se inserta
            Key::Char(c) => editor.insert_char(c),

            _ => {}
        }

        editor.adjust_scroll();
        editor.write(&mut stdout);
    }

    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();
}
