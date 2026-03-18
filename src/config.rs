use crossterm::style::Color;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

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
    let theme = SyntaxTheme::default();

    let Some(path) = find_config_path() else {
        return theme;
    };

    load_syntax_theme_from_path(&path).unwrap_or(theme)
}

fn load_syntax_theme_from_path(path: &Path) -> Option<SyntaxTheme> {
    let content = fs::read_to_string(path).ok()?;
    parse_syntax_theme(&content)
}

fn parse_syntax_theme(content: &str) -> Option<SyntaxTheme> {
    let mut theme = SyntaxTheme::default();
    let mut in_syntax_section = false;
    let mut parsed_any = false;

    for raw_line in content.lines() {
        let line = strip_inline_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len() - 1].trim();
            in_syntax_section = *section == "syntax";
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

        match (in_syntax_section, key) {
            (true, "keyword") | (_, "syntax.keyword") => {
                theme.keyword = color;
                parsed_any = true;
            }
            (true, "string") | (_, "syntax.string") => {
                theme.string = color;
                parsed_any = true;
            }
            (true, "number") | (_, "syntax.number") => {
                theme.number = color;
                parsed_any = true;
            }
            (true, "comment") | (_, "syntax.comment") => {
                theme.comment = color;
                parsed_any = true;
            }
            _ => {}
        }
    }

    parsed_any.then_some(theme)
}

fn strip_inline_comment(line: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut prev_was_escape = false;

    for (idx, ch) in line.char_indices() {
        match ch {
            '\'' if !in_double && !prev_was_escape => in_single = !in_single,
            '"' if !in_single && !prev_was_escape => in_double = !in_double,
            '#' if !in_single && !in_double => return &line[..idx],
            _ => {}
        }

        prev_was_escape = ch == '\\' && !prev_was_escape;
        if ch != '\\' {
            prev_was_escape = false;
        }
    }

    line
}

fn find_config_path() -> Option<PathBuf> {
    let current_dir = env::current_dir().ok();
    let env_config = env::var_os("HYPERION_CONFIG").map(PathBuf::from);
    let xdg_config_home = env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    let home_dir = env::var_os("HOME").map(PathBuf::from);

    find_config_path_with(current_dir, env_config, xdg_config_home, home_dir)
}

fn find_config_path_with(
    current_dir: Option<PathBuf>,
    env_config: Option<PathBuf>,
    xdg_config_home: Option<PathBuf>,
    home_dir: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(path) = env_config {
        let resolved = if path.is_relative() {
            current_dir
                .as_ref()
                .map(|dir| dir.join(&path))
                .unwrap_or(path.clone())
        } else {
            path.clone()
        };

        if resolved.is_file() {
            return Some(resolved);
        }
    }

    if let Some(dir) = current_dir.as_ref() {
        for candidate in [".hyperion.toml", "hyperion.toml"] {
            let path = dir.join(candidate);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    if let Some(dir) = xdg_config_home {
        let path = dir.join("hyperion/config.toml");
        if path.is_file() {
            return Some(path);
        }
    }

    if let Some(home) = home_dir {
        let path = home.join(".config/hyperion/config.toml");
        if path.is_file() {
            return Some(path);
        }
    }

    None
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
    use std::time::{SystemTime, UNIX_EPOCH};

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
    fn parse_theme_from_syntax_section() {
        let content = r##"
            [syntax]
            keyword = "#112233"
            string = "#445566"
            number = "#778899"
            comment = "#AABBCC"
        "##;

        let theme = parse_syntax_theme(content).unwrap();

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

    #[test]
    fn parse_theme_with_inline_comments() {
        let content = r##"
            [syntax]
            keyword = "#112233" # comentario
            string = "#445566"
        "##;

        let theme = parse_syntax_theme(content).unwrap();

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
    }

    #[test]
    fn parse_theme_from_dotted_keys() {
        let content = r##"
            syntax.keyword = "#010203"
            syntax.comment = "#0A0B0C"
        "##;

        let theme = parse_syntax_theme(content).unwrap();

        assert_eq!(theme.keyword, Color::Rgb { r: 1, g: 2, b: 3 });
        assert_eq!(
            theme.comment,
            Color::Rgb {
                r: 10,
                g: 11,
                b: 12
            }
        );
    }

    #[test]
    fn find_config_path_respects_precedence() {
        let base = unique_temp_dir();
        let current_dir = base.join("cwd");
        let config_home = base.join("xdg");
        let home_dir = base.join("home");
        fs::create_dir_all(&current_dir).unwrap();
        fs::create_dir_all(config_home.join("hyperion")).unwrap();
        fs::create_dir_all(home_dir.join(".config/hyperion")).unwrap();

        let env_path = current_dir.join("custom.toml");
        let local_path = current_dir.join(".hyperion.toml");
        let xdg_path = config_home.join("hyperion/config.toml");
        let home_path = home_dir.join(".config/hyperion/config.toml");

        fs::write(&env_path, "[syntax]\nkeyword = \"#123456\"\n").unwrap();
        fs::write(&local_path, "[syntax]\nkeyword = \"#234567\"\n").unwrap();
        fs::write(&xdg_path, "[syntax]\nkeyword = \"#345678\"\n").unwrap();
        fs::write(&home_path, "[syntax]\nkeyword = \"#456789\"\n").unwrap();

        let path = find_config_path_with(
            Some(current_dir),
            Some(env_path.clone()),
            Some(config_home),
            Some(home_dir),
        );

        assert_eq!(path, Some(env_path));
    }

    #[test]
    fn load_theme_from_selected_path() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).unwrap();
        let path = base.join("hyperion.toml");
        fs::write(
            &path,
            "[syntax]\nkeyword = \"#010203\"\ncomment = \"#0A0B0C\"\n",
        )
        .unwrap();

        let theme = load_syntax_theme_from_path(&path).unwrap();

        assert_eq!(theme.keyword, Color::Rgb { r: 1, g: 2, b: 3 });
        assert_eq!(
            theme.comment,
            Color::Rgb {
                r: 10,
                g: 11,
                b: 12
            }
        );
        assert_eq!(theme.string, Color::Green);
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("hyperion_test_{nanos}"))
    }
}
