use crossterm::style::Color;
use serde::Deserialize;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiTheme {
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub tab_bar_bg: Color,
    pub tab_bar_fg: Color,
    pub selection_bg: Color,
}

impl UiTheme {
    pub fn default_theme() -> Self {
        Self {
            status_bar_bg: Color::White,
            status_bar_fg: Color::Black,
            tab_bar_bg: Color::DarkGrey,
            tab_bar_fg: Color::White,
            selection_bg: Color::DarkBlue,
        }
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::default_theme()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageConfig {
    pub rust: Vec<String>,
    pub python: Vec<String>,
    pub javascript: Vec<String>,
}

impl LanguageConfig {
    pub fn default_config() -> Self {
        Self {
            rust: vec!["rs".to_string()],
            python: vec!["py".to_string()],
            javascript: vec![
                "js".to_string(),
                "mjs".to_string(),
                "cjs".to_string(),
                "ts".to_string(),
            ],
        }
    }
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub show_line_numbers: bool,
}

impl EditorConfig {
    pub fn default_config() -> Self {
        Self {
            tab_size: 4,
            show_line_numbers: true,
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self::default_theme()
    }
}

#[derive(Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    syntax: RawSyntaxTheme,
    #[serde(default)]
    editor: RawEditorConfig,
    #[serde(default)]
    ui: RawUiTheme,
    #[serde(default)]
    languages: RawLanguageConfig,
}
#[derive(Deserialize, Default)]
struct RawUiTheme {
    status_bar_bg: Option<String>,
    status_bar_fg: Option<String>,
    tab_bar_bg: Option<String>,
    tab_bar_fg: Option<String>,
    selection_bg: Option<String>,
}
#[derive(Deserialize, Default)]
struct RawLanguageConfig {
    rust: Option<Vec<String>>,
    python: Option<Vec<String>>,
    javascript: Option<Vec<String>>,
}
#[derive(Deserialize, Default)]
struct RawSyntaxTheme {
    keyword: Option<String>,
    string: Option<String>,
    number: Option<String>,
    comment: Option<String>,
}

#[derive(Deserialize, Default)]
struct RawEditorConfig {
    tab_size: Option<usize>,
    show_line_numbers: Option<bool>,
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
    let raw: RawConfig = toml::from_str(content).ok()?;
    let mut theme = SyntaxTheme::default();
    let mut parsed_any = false;

    if let Some(color) = raw.syntax.keyword.as_deref().and_then(parse_hex_color) {
        theme.keyword = color;
        parsed_any = true;
    }
    if let Some(color) = raw.syntax.string.as_deref().and_then(parse_hex_color) {
        theme.string = color;
        parsed_any = true;
    }
    if let Some(color) = raw.syntax.number.as_deref().and_then(parse_hex_color) {
        theme.number = color;
        parsed_any = true;
    }
    if let Some(color) = raw.syntax.comment.as_deref().and_then(parse_hex_color) {
        theme.comment = color;
        parsed_any = true;
    }

    parsed_any.then_some(theme)
}

pub fn load_ui_theme() -> UiTheme {
    let theme = UiTheme::default();

    let Some(path) = find_config_path() else {
        return theme;
    };

    load_ui_theme_from_path(&path).unwrap_or(theme)
}

fn load_ui_theme_from_path(path: &Path) -> Option<UiTheme> {
    let content = fs::read_to_string(path).ok()?;
    parse_ui_theme(&content)
}

fn parse_ui_theme(content: &str) -> Option<UiTheme> {
    let raw: RawConfig = toml::from_str(content).ok()?;
    let mut theme = UiTheme::default();
    let mut parsed_any = false;

    if let Some(color) = raw.ui.status_bar_bg.as_deref().and_then(parse_hex_color) {
        theme.status_bar_bg = color;
        parsed_any = true;
    }
    if let Some(color) = raw.ui.status_bar_fg.as_deref().and_then(parse_hex_color) {
        theme.status_bar_fg = color;
        parsed_any = true;
    }
    if let Some(color) = raw.ui.tab_bar_bg.as_deref().and_then(parse_hex_color) {
        theme.tab_bar_bg = color;
        parsed_any = true;
    }
    if let Some(color) = raw.ui.tab_bar_fg.as_deref().and_then(parse_hex_color) {
        theme.tab_bar_fg = color;
        parsed_any = true;
    }
    if let Some(color) = raw.ui.selection_bg.as_deref().and_then(parse_hex_color) {
        theme.selection_bg = color;
        parsed_any = true;
    }

    parsed_any.then_some(theme)
}
pub fn load_language_config() -> LanguageConfig {
    let config = LanguageConfig::default();

    let Some(path) = find_config_path() else {
        return config;
    };

    load_language_config_from_path(&path).unwrap_or(config)
}

fn load_language_config_from_path(path: &Path) -> Option<LanguageConfig> {
    let content = fs::read_to_string(path).ok()?;
    parse_language_config(&content)
}

fn parse_language_config(content: &str) -> Option<LanguageConfig> {
    let raw: RawConfig = toml::from_str(content).ok()?;
    let defaults = LanguageConfig::default();

    if raw.languages.rust.is_none()
        && raw.languages.python.is_none()
        && raw.languages.javascript.is_none()
    {
        return None;
    }

    Some(LanguageConfig {
        rust: raw.languages.rust.unwrap_or(defaults.rust),
        python: raw.languages.python.unwrap_or(defaults.python),
        javascript: raw.languages.javascript.unwrap_or(defaults.javascript),
    })
}
pub fn load_editor_config() -> EditorConfig {
    let config = EditorConfig::default();

    let Some(path) = find_config_path() else {
        return config;
    };

    load_editor_config_from_path(&path).unwrap_or(config)
}

fn load_editor_config_from_path(path: &Path) -> Option<EditorConfig> {
    let content = fs::read_to_string(path).ok()?;
    parse_editor_config(&content)
}

fn parse_editor_config(content: &str) -> Option<EditorConfig> {
    let raw: RawConfig = toml::from_str(content).ok()?;
    let defaults = EditorConfig::default();

    let tab_size = match raw.editor.tab_size {
        Some(0) | None => defaults.tab_size,
        Some(size) => size,
    };
    let show_line_numbers = raw
        .editor
        .show_line_numbers
        .unwrap_or(defaults.show_line_numbers);

    if raw.editor.tab_size.is_none() && raw.editor.show_line_numbers.is_none() {
        return None;
    }

    Some(EditorConfig {
        tab_size,
        show_line_numbers,
    })
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
        let expanded = expand_home_path(&path, home_dir.as_deref()).unwrap_or(path);
        let resolved = if expanded.is_relative() {
            current_dir
                .as_ref()
                .map(|dir| dir.join(&expanded))
                .unwrap_or(expanded.clone())
        } else {
            expanded.clone()
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

fn expand_home_path(path: &Path, home_dir: Option<&Path>) -> Option<PathBuf> {
    let home_dir = home_dir?;
    let raw = path.to_str()?;

    if raw == "~" {
        return Some(home_dir.to_path_buf());
    }

    let suffix = raw.strip_prefix("~/")?;
    Some(home_dir.join(suffix))
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
    fn expand_home_path_for_env_var() {
        let home_dir = PathBuf::from("/tmp/hyperion-home");
        let expanded =
            expand_home_path(Path::new("~/.config/hyperion/config.toml"), Some(&home_dir));

        assert_eq!(
            expanded,
            Some(home_dir.join(".config/hyperion/config.toml"))
        );
    }

    #[test]
    fn find_config_path_expands_tilde_from_env_var() {
        let base = unique_temp_dir();
        let current_dir = base.join("cwd");
        let home_dir = base.join("home");
        fs::create_dir_all(&current_dir).unwrap();
        fs::create_dir_all(home_dir.join(".config/hyperion")).unwrap();

        let home_path = home_dir.join(".config/hyperion/config.toml");
        fs::write(&home_path, "[syntax]\nkeyword = \"#ABCDEF\"\n").unwrap();

        let path = find_config_path_with(
            Some(current_dir),
            Some(PathBuf::from("~/.config/hyperion/config.toml")),
            None,
            Some(home_dir.clone()),
        );

        assert_eq!(path, Some(home_path));
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
    #[test]
    fn parse_editor_config_tab_size() {
        let content = "[editor]\ntab_size = 2\n";
        let config = parse_editor_config(content).unwrap();
        assert_eq!(config.tab_size, 2);
    }

    #[test]
    fn parse_editor_config_ignores_zero() {
        let content = "[editor]\ntab_size = 0\n";
        let config = parse_editor_config(content).unwrap();
        assert_eq!(config.tab_size, 4);
    }
    #[test]
    fn parse_editor_config_show_line_numbers() {
        let content = "[editor]\nshow_line_numbers = false\n";
        let config = parse_editor_config(content).unwrap();
        assert!(!config.show_line_numbers);
        assert_eq!(config.tab_size, 4); // default, no especificado
    }

    #[test]
    fn parse_ui_theme_from_section() {
        let content = r##"
            [ui]
            status_bar_bg = "#FFFFFF"
            selection_bg = "#123456"
        "##;

        let theme = parse_ui_theme(content).unwrap();

        assert_eq!(
            theme.status_bar_bg,
            Color::Rgb {
                r: 255,
                g: 255,
                b: 255
            }
        );
        assert_eq!(
            theme.selection_bg,
            Color::Rgb {
                r: 18,
                g: 52,
                b: 86
            }
        );
        assert_eq!(theme.tab_bar_bg, Color::DarkGrey); // default, no especificado
    }
    #[test]
    fn parse_language_config_custom_extensions() {
        let content = r##"
            [languages]
            rust = ["rs", "rslib"]
            javascript = ["js", "tsx"]
        "##;

        let config = parse_language_config(content).unwrap();

        assert_eq!(config.rust, vec!["rs", "rslib"]);
        assert_eq!(config.javascript, vec!["js", "tsx"]);
        assert_eq!(config.python, vec!["py"]); // default, no especificado
    }
}
