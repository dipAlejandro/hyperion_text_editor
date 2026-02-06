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
