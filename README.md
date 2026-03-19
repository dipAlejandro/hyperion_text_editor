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
hyperion archivo.txt
```

------------------------------------------------------------------------

## 📜 Licencia

MIT

- `Ctrl+Q` - Salir
- `Ctrl+S` - Guardar
- `Ctrl+O` - Abrir
- `Ctrl+F` - Buscar
- `Ctrl+N` - Siguiente resultado
- `Ctrl+P` - Resultado anterior
- `Ctrl+G` - Ir a línea
- Flechas - Navegar
- `Tab` - Insertar 4 espacios
- `Home` / `End` - Ir al inicio / final de la línea
- `Page Up` / `Page Down` - Mover una página
- `Delete` / `Backspace` - Borrar carácter siguiente / anterior

## Características

- ✨ Soporte UTF-8 completo
- 🔍 Búsqueda con resaltado
- 📝 Números de línea
- 🎯 Scroll automático
- ⚡ Rápido y ligero

## Configuración de colores de sintaxis

Podes personalizar los colores de sintaxis creando un archivo de configuración en alguno de estos paths (en orden de prioridad):

1. Ruta indicada por la variable de entorno `HYPERION_CONFIG`
2. `./.hyperion.toml`
3. `./hyperion.toml`
4. `$XDG_CONFIG_HOME/hyperion/config.toml`
5. `~/.config/hyperion/config.toml`

Ejemplo:

```toml
[syntax]
keyword = "#569CD6"
string = "#98C379"
number = "#E5C07B"
comment = "#5C6370"
```

Si usas `HYPERION_CONFIG`, puedes apuntar tanto a una ruta absoluta como a una ruta con `~`, por ejemplo:

```bash
export HYPERION_CONFIG="~/.config/hyperion/config.toml"
```
