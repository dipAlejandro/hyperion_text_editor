use crossterm::style::Color;
use std::{env, fs, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxTheme {
    pub keyword: Color,
    pub string: Color,
    pub number: Color,
    pub comment: Color,
}

impl SyntaxTheme {
    pub fn default_theme() -> Self {
        Self {
            keyword: Color::Blue,
            string: Color::Green,
            number: Color::Yellow,
            comment: Color::DarkGrey,
        }
    }
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self::default_theme()
    }
}

pub fn load_syntax_theme() -> SyntaxTheme {
    let mut theme = SyntaxTheme::default();

    let Some(path) = find_config_path() else {
        return theme;
    };

    let Ok(content) = fs::read_to_string(path) else {
        return theme;
    };

    apply_theme_from_str(&content, &mut theme);
    theme
}

fn find_config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("HYPERION_CONFIG") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    let local_candidates = [
        PathBuf::from(".hyperion.toml"),
        PathBuf::from("hyperion.toml"),
    ];
    for candidate in local_candidates {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    if let Ok(home) = env::var("HOME") {
        let path = PathBuf::from(home).join(".config/hyperion/config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn apply_theme_from_str(content: &str, theme: &mut SyntaxTheme) {
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("//")
            || line.starts_with('[')
        {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let key = key.trim();
        let value = value.trim().trim_matches('"').trim_matches('\'');

        let Some(color) = parse_hex_color(value) else {
            continue;
        };

        match key {
            "keyword" | "syntax.keyword" => theme.keyword = color,
            "string" | "syntax.string" => theme.string = color,
            "number" | "syntax.number" => theme.number = color,
            "comment" | "syntax.comment" => theme.comment = color,
            _ => {}
        }
    }
}

pub fn parse_hex_color(input: &str) -> Option<Color> {
    let hex = input.strip_prefix('#').unwrap_or(input);
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb { r, g, b })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_hex_color() {
        assert_eq!(
            parse_hex_color("#FF00AA"),
            Some(Color::Rgb {
                r: 255,
                g: 0,
                b: 170
            })
        );
    }

    #[test]
    fn parse_invalid_hex_color() {
        assert_eq!(parse_hex_color("#GG00AA"), None);
        assert_eq!(parse_hex_color("#123"), None);
    }

    #[test]
    fn apply_theme_from_config_lines() {
        let mut theme = SyntaxTheme::default();
        let content = r##"
            [syntax]
            keyword = "#112233"
            syntax.string = "#445566"
            number = "#778899"
            comment = "#AABBCC"
        "##;

        apply_theme_from_str(content, &mut theme);

        assert_eq!(
            theme.keyword,
            Color::Rgb {
                r: 17,
                g: 34,
                b: 51
            }
        );
        assert_eq!(
            theme.string,
            Color::Rgb {
                r: 68,
                g: 85,
                b: 102
            }
        );
        assert_eq!(
            theme.number,
            Color::Rgb {
                r: 119,
                g: 136,
                b: 153
            }
        );
        assert_eq!(
            theme.comment,
            Color::Rgb {
                r: 170,
                g: 187,
                b: 204
            }
        );
    }
}
