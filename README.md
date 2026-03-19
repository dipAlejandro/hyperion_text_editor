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

## Atajos de Teclado

- `Ctrl+Q` - Salir
- `Ctrl+S` - Guardar
- `Ctrl+O` - Abrir
- `Ctrl+F` - Buscar
- `Ctrl+N` - Siguiente resultado
- `Ctrl+P` - Resultado anterior
- `Ctrl+G` - Ir a línea
- Flechas - Navegar

## Características

- ✨ Soporte UTF-8 completo
- 🔍 Búsqueda con resaltado
- 📝 Números de línea
- 🎯 Scroll automático
- ⚡ Rápido y ligero

## Configuración de colores de sintaxis

Puedes personalizar los colores de sintaxis creando un archivo de configuración en alguno de estos paths (en orden de prioridad):

1. Ruta indicada por la variable de entorno `HYPERION_CONFIG`
2. `./.hyperion.toml`
3. `./hyperion.toml`
4. `~/.config/hyperion/config.toml`

Ejemplo:

```toml
[syntax]
keyword = "#569CD6"
string = "#98C379"
number = "#E5C07B"
comment = "#5C6370"
```
