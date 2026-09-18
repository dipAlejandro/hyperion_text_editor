# Contribuir a Hyperion Text Editor

¡Gracias por tu interés en contribuir! Esta guía explica cómo reportar bugs y proponer cambios.

------------------------------------------------------------------------

## 🐛 Reportar bugs

¿Encontraste un bug? Reportalo abriendo un [Issue](https://github.com/dipAlejandro/hyperion_text_editor/issues/new).

### Antes de reportar

- Revisá los [issues abiertos](https://github.com/dipAlejandro/hyperion_text_editor/issues) para evitar duplicados.
- Confirmá que estás usando la última versión (`hy --version` o el binario más reciente de [Releases](https://github.com/dipAlejandro/hyperion_text_editor/releases)).

### Qué incluir en el reporte

- **Pasos para reproducir** el problema (lo más específico posible).
- **Comportamiento esperado** vs **comportamiento actual**.
- **Sistema operativo** y terminal usada (ej: Linux + Alacritty, Windows + PowerShell).
- Si aplica, el **archivo o contenido** que causó el problema (o un ejemplo mínimo).
- Cualquier mensaje de error o captura de pantalla.

------------------------------------------------------------------------

## 🔧 Contribuir con código

1. Hacé fork del repositorio.
2. Creá una rama descriptiva desde `develop`: `git checkout -b fix/nombre-del-bug develop`.
3. Asegurate de que `cargo test` y `cargo build` pasen sin errores ni warnings.
4. Abrí un Pull Request **contra `develop`** (no `main`) describiendo el cambio y, si corresponde, el issue que resuelve.

`main` refleja únicamente versiones publicadas (releases); todo el desarrollo activo pasa por `develop` antes de llegar ahí.

------------------------------------------------------------------------

## 🎯 Áreas donde ayuda extra es bienvenida

- Testeo en archivos grandes (rendimiento, scroll).
- Compatibilidad en Windows/macOS (el desarrollo principal es en Linux).
- Casos límite de UTF-8 / caracteres especiales.
- Configuración TOML con valores inválidos o edge cases.