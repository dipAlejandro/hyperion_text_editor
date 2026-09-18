# Hyperion Text Editor

**Hyperion** es un editor de texto minimalista para la terminal, escrito
en **Rust**, enfocado en simplicidad de uso, similar a nano.

------------------------------------------------------------------------

## 📦 Instalación

> Actualmente soporta **Linux, macOS y Windows (x86_64)**.

### Opción 1: Descargar binarios (recomendado)

https://github.com/dipAlejandro/hyperion_text_editor/releases

Descargá el archivo correspondiente a tu sistema:

-   Linux: `hyperion-x86_64-unknown-linux-gnu.tar.gz`
-   macOS Intel: `hyperion-x86_64-apple-darwin.tar.gz`
-   macOS Apple Silicon: `hyperion-aarch64-apple-darwin.tar.gz`
-   Windows: `hyperion-x86_64-pc-windows-msvc.zip`

#### Instalación (Linux / macOS)

Extraer el binario:

``` bash
tar -xzf hyperion-*.tar.gz
```

Mover el ejecutable a un directorio del `PATH`:

``` bash
sudo mv hyperion /usr/local/bin/
sudo chmod +x /usr/local/bin/hyperion
```

Verificar instalación:

``` bash
hyperion --help
```

------------------------------------------------------------------------

#### Instalación (Windows)
Extraé el archivo `.zip`. Obtendrás `hyperion.exe`.

##### Opción A (rápida)

``` powershell
.\hyperion.exe --help
```

##### Opción B (recomendada)

-   Crear carpeta: `C:\Tools\hyperion`
-   Mover `hyperion.exe`
-   Agregar la carpeta al `PATH`

Verificar:

``` powershell
hyperion --help
```

------------------------------------------------------------------------

### Opción 2: Compilar desde el código fuente

Requisitos: - Rust (stable) → https://rustup.rs

``` bash
git clone https://github.com/dipAlejandro/hyperion_text_editor.git
cd hyperion_text_editor
cargo build --release
```

------------------------------------------------------------------------

## Uso

``` bash
hy archivo.txt
```
----------------------------------------------------------------------

## 📜 Licencia

MIT

Sin argumentos, abre el editor con un buffer nuevo sin nombre.

------------------------------------------------------------------------

## Atajos de teclado

### Archivo
- `Ctrl+Q` - Salir (doble `Ctrl+Q` si hay cambios sin guardar)
- `Ctrl+S` - Guardar
- `Ctrl+O` - Abrir archivo (file picker, ver detalle abajo)

### Pestañas (multi-buffer)
- `Ctrl+T` - Nueva pestaña
- `Ctrl+W` - Cerrar pestaña actual
- `Alt+←` / `Alt+→` - Pestaña anterior / siguiente

### Búsqueda y reemplazo
- `Ctrl+F` - Buscar
- `Ctrl+N` - Siguiente resultado
- `Ctrl+P` - Resultado anterior
- `Ctrl+R` - Reemplazar coincidencia actual (pide el texto de reemplazo la primera vez)
- `Ctrl+Shift+R` - Reemplazar todas las coincidencias
- `Ctrl+G` - Ir a línea

### Edición
- `Ctrl+Z` - Deshacer
- `Ctrl+Y` / `Ctrl+Shift+Z` - Rehacer
- `Ctrl+A` - Seleccionar todo
- `Ctrl+X` - Cortar selección
- `Ctrl+C` - Copiar selección
- `Ctrl+Shift+C` - Copiar línea completa
- `Ctrl+V` - Pegar
- `Tab` - Insertar espacios (cantidad configurable, ver `tab_size`)
- `Enter` - Nueva línea (con auto-indentación)
- `Delete` / `Backspace` - Borrar carácter siguiente / anterior (borra la selección si hay una)

### Navegación
- Flechas - Mover el cursor
- `Shift` + Flechas - Seleccionar texto
- `Home` / `End` - Ir al inicio / final de la línea
- `Page Up` / `Page Down` - Mover una página
- Click izquierdo (mouse) - Posicionar el cursor
- Rueda del mouse - Scroll

------------------------------------------------------------------------

## 🔎 File picker (Ctrl+O)

Al presionar `Ctrl+O` se abre un explorador de directorios con búsqueda difusa (fuzzy):

- Muestra el contenido del directorio actual (dotfiles ocultos), con `..` como primera entrada para subir al padre.
- Escribí para filtrar por nombre; los caracteres coincidentes se resaltan.
- `↑` / `↓` - Mover la selección
- `Enter` - Entrar al directorio seleccionado, o abrir el archivo seleccionado
- `Backspace` - Borrar el último carácter del filtro
- `Esc` - Cancelar

El prompt superior muestra la ruta del directorio actual.

------------------------------------------------------------------------

## Características

- ✨ Soporte UTF-8 completo
- 🔍 Búsqueda y reemplazo con resaltado
- 🔎 File picker con búsqueda difusa (Ctrl+O)
- 📝 Números de línea (configurable)
- 📑 Múltiples pestañas / buffers
- 🖱️ Soporte de mouse (click y scroll)
- ↩️ Undo / Redo
- 🎯 Scroll automático
- 🎨 Resaltado de sintaxis (Rust, Python, JavaScript/TypeScript)
- ⚙️ Configuración personalizable vía TOML
- ⚡ Rápido y ligero

------------------------------------------------------------------------

## ⚙️ Configuración

Podés personalizar el editor creando un archivo TOML en alguno de estos
paths (en orden de prioridad):

1. Ruta indicada por la variable de entorno `HYPERION_CONFIG`
2. `./.hyperion.toml`
3. `./hyperion.toml`
4. `$XDG_CONFIG_HOME/hyperion/config.toml`
5. `~/.config/hyperion/config.toml`

Si usas `HYPERION_CONFIG`, puedes apuntar tanto a una ruta absoluta como
a una ruta con `~`, por ejemplo:

```bash
export HYPERION_CONFIG="~/.config/hyperion/config.toml"
```

Todas las secciones son opcionales; lo que no se especifica usa su
valor por defecto.

### Ejemplo completo

```toml
[syntax]
keyword = "#569CD6"
string = "#98C379"
number = "#E5C07B"
comment = "#5C6370"

[editor]
tab_size = 4
show_line_numbers = true

[ui]
status_bar_bg = "#FFFFFF"
status_bar_fg = "#000000"
tab_bar_bg = "#444444"
tab_bar_fg = "#FFFFFF"
selection_bg = "#00008B"

[languages]
rust = ["rs"]
python = ["py", "pyi"]
javascript = ["js", "mjs", "cjs", "ts", "tsx"]
```

### Referencia de opciones

| Sección     | Clave              | Tipo           | Default                          | Descripción                                  |
|-------------|--------------------|----------------|-----------------------------------|-----------------------------------------------|
| `syntax`    | `keyword`          | color hex      | `#569CD6` (azul)                 | Color de palabras clave                       |
| `syntax`    | `string`            | color hex      | verde                            | Color de strings                              |
| `syntax`    | `number`            | color hex      | amarillo                         | Color de números                              |
| `syntax`    | `comment`           | color hex      | gris oscuro                      | Color de comentarios                          |
| `editor`    | `tab_size`          | entero > 0     | `4`                               | Cantidad de espacios que inserta `Tab`        |
| `editor`    | `show_line_numbers` | booleano       | `true`                            | Mostrar/ocultar números de línea              |
| `ui`        | `status_bar_bg`     | color hex      | blanco                           | Fondo de la barra de estado                   |
| `ui`        | `status_bar_fg`     | color hex      | negro                             | Texto de la barra de estado                   |
| `ui`        | `tab_bar_bg`        | color hex      | gris oscuro                       | Fondo de la barra de pestañas                 |
| `ui`        | `tab_bar_fg`        | color hex      | blanco                            | Texto de la barra de pestañas                 |
| `ui`        | `selection_bg`      | color hex      | azul oscuro                       | Fondo del texto seleccionado                  |
| `languages` | `rust`              | lista de strings | `["rs"]`                       | Extensiones asociadas a resaltado Rust        |
| `languages` | `python`            | lista de strings | `["py"]`                       | Extensiones asociadas a resaltado Python      |
| `languages` | `javascript`        | lista de strings | `["js","mjs","cjs","ts"]`     | Extensiones asociadas a resaltado JavaScript  |

Los colores se especifican en formato hexadecimal, con o sin `#`:

```toml
keyword = "#569CD6"
keyword = "569CD6"   # también válido
```

`tab_size = 0` se ignora y se usa el valor por defecto (`4`).

------------------------------------------------------------------------

## 📜 Licencia

MIT

------------------------------------------------------------------------

## 🐛 Contribuir

¿Encontraste un bug o querés proponer un cambio? Mirá la guía en [CONTRIBUTING.md](./CONTRIBUTING.md).