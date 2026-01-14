# Hyperion Text Editor

**Hyperion** es un editor de texto minimalista para la terminal, escrito en **Rust**, enfocado en simplicidad de uso, similar a nano.

---

---

## 📦 Instalación

> ⚠️ Actualmente soporta **Linux y macOS**.\
> [!abstract]

### Opción 1: Descargar binarios (recomendado)

https://github.com/dipAlejandro/hyperion_text_editor/releases

Descargá el archivo correspondiente a tu sistema:

- Linux: `hyperion-x86_64-unknown-linux-gnu.tar.gz`
- macOS Intel: `hyperion-x86_64-apple-darwin.tar.gz`
- macOS Apple Silicon: `hyperion-aarch64-apple-darwin.tar.gz`

Extraer el binario:

```bash
tar -xzf hyperion-*.tar.gz
```

Mover el ejecutable a un directorio del `PATH` (Linux y macOS):

```bash
sudo mv hyperion /usr/local/bin/
sudo chmod +x /usr/local/bin/hyperion
```

Verificar instalación:

```bash
hyperion --help
```

---

### Opción 2: Compilar desde el código fuente

Requisitos:

- Rust (stable) → [https://rustup.rs](https://rustup.rs)

```bash
git clone https://github.com/dipAlejandro/hyperion_text_editor.git
cd hyperion_text_editor
cargo build --release
```

Instalar el binario compilado:

```bash
sudo cp target/release/hyperion /usr/local/bin/
```

---

## Uso

```bash
hyperion archivo.txt
```

Si el archivo no existe, se crea automáticamente.

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

---

## 📄 Estado del proyecto

🟡 **En desarrollo activo**

- El formato de archivo es estable
- La API interna puede cambiar
- No hay compatibilidad garantizada entre versiones tempranas

---

## 🤝 Contribuciones

Las contribuciones son bienvenidas.

Si querés colaborar:

1. Fork del repositorio
1. Crear una rama (`feature/lo-que-sea`)
1. Commits claros y pequeños
1. Pull Request con explicación

---

## 📜 Licencia

Este proyecto está bajo la licencia **MIT**.

````

# Hyperion 🚀

Editor de texto minimalista para terminal escrito en Rust.

## Instalación
```bash
cargo install --path .
````

## Uso

```bash
# Abrir archivo
hyperion archivo.txt

# Crear archivo nuevo
hyperion nuevo.py

# Editor vacío
hyperion
```
