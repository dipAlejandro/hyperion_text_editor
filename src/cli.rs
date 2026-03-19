//! Módulo de parsing de argumentos de línea de comandos

use clap::Parser;

/// Editor de texto simple en terminal
#[derive(Parser, Debug)]
#[command(name = "hyperion")]
#[command(author = "Alejandro Dip")]
#[command(version = "1.0.0")]
#[command(about = "Editor de texto minimalista para terminal", long_about = None)]
pub struct Args {
    /// Archivo a abrir o crear
    #[arg(value_name = "FILE")]
    pub file: Option<String>,
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}
