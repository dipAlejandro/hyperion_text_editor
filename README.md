# Hyperion 🚀

Editor de texto minimalista para terminal escrito en Rust.

## Instalación
```bash
cargo install --path .
```

## Uso
```bash
# Abrir archivo
hyperion archivo.txt

# Crear archivo nuevo
hyperion nuevo.py

# Editor vacío
hyperion
```

## Atajos de teclado

- `Ctrl+Q` - Salir (doble Ctrl+Q si hay cambios sin guardar)
- `Ctrl+S` - Guardar
- `Ctrl+O` - Abrir
- `Ctrl+F` - Buscar
- `Ctrl+R` - Reemplazar coincidencia actual
- `Ctrl+Shift+R` - Reemplazar todas las coincidencias
- `Ctrl+N` - Siguiente resultado
- `Ctrl+P` - Resultado anterior
- `Ctrl+G` - Ir a línea
- `Ctrl+Z` - Deshacer
- `Ctrl+Y` / `Ctrl+Shift+Z` - Rehacer
- `Ctrl+A` - Seleccionar todo
- `Ctrl+X` - Cortar selección
- `Ctrl+C` - Copiar selección
- `Ctrl+Shift+C` - Copiar línea completa
- `Ctrl+V` - Pegar
- Flechas - Navegar
- `Shift` + Flechas - Seleccionar texto
- `Tab` - Insertar 4 espacios
- `Enter` - Nueva línea (con auto-indentación)
- `Home` / `End` - Ir al inicio / final de la línea
- `Page Up` / `Page Down` - Mover una página
- `Delete` / `Backspace` - Borrar carácter siguiente / anterior (borra la selección si hay una)

## Características

- ⚡ Rápido y ligero
- ✨ Soporte UTF-8 completo
- 🔍 Búsqueda con resaltado
- 📝 Números de línea
- 🎯 Scroll automático

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
