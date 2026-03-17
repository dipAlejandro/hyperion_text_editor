use crossterm::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntaxLanguage {
    Rust,
    Python,
    JavaScript,
    PlainText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Keyword,
    String,
    Number,
    Comment,
}

impl TokenKind {
    pub fn color(self) -> Color {
        match self {
            TokenKind::Keyword => Color::Blue,
            TokenKind::String => Color::Green,
            TokenKind::Number => Color::Yellow,
            TokenKind::Comment => Color::DarkGrey,
        }
    }
}

pub fn detect_language(filename: Option<&str>) -> SyntaxLanguage {
    let Some(filename) = filename else {
        return SyntaxLanguage::PlainText;
    };

    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    match ext.as_deref() {
        Some("rs") => SyntaxLanguage::Rust,
        Some("py") => SyntaxLanguage::Python,
        Some("js") | Some("mjs") | Some("cjs") | Some("ts") => SyntaxLanguage::JavaScript,
        _ => SyntaxLanguage::PlainText,
    }
}

pub fn tokenize_line(line: &str, language: SyntaxLanguage) -> Vec<Option<TokenKind>> {
    let chars: Vec<char> = line.chars().collect();
    let mut tokens = vec![None; chars.len()];

    if language == SyntaxLanguage::PlainText {
        return tokens;
    }

    let mut i = 0;
    while i < chars.len() {
        if starts_comment(&chars, i, language) {
            for t in &mut tokens[i..] {
                *t = Some(TokenKind::Comment);
            }
            break;
        }

        let ch = chars[i];

        if ch == '"' || ch == '\'' {
            let quote = ch;
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if chars[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            for token in tokens.iter_mut().take(i.min(chars.len())).skip(start) {
                *token = Some(TokenKind::String);
            }
            continue;
        }

        if ch.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_') {
                i += 1;
            }
            for token in tokens.iter_mut().take(i).skip(start) {
                *token = Some(TokenKind::Number);
            }
            continue;
        }

        if is_ident_start(ch) {
            let start = i;
            i += 1;
            while i < chars.len() && is_ident_continue(chars[i]) {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();
            if is_keyword(&ident, language) {
                for token in tokens.iter_mut().take(i).skip(start) {
                    *token = Some(TokenKind::Keyword);
                }
            }
            continue;
        }

        i += 1;
    }

    tokens
}

fn starts_comment(chars: &[char], i: usize, language: SyntaxLanguage) -> bool {
    match language {
        SyntaxLanguage::Rust | SyntaxLanguage::JavaScript => {
            i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/'
        }
        SyntaxLanguage::Python => chars[i] == '#',
        SyntaxLanguage::PlainText => false,
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_keyword(word: &str, language: SyntaxLanguage) -> bool {
    match language {
        SyntaxLanguage::Rust => matches!(
            word,
            "fn" | "let"
                | "mut"
                | "if"
                | "else"
                | "match"
                | "for"
                | "while"
                | "loop"
                | "struct"
                | "enum"
                | "impl"
                | "trait"
                | "pub"
                | "use"
                | "mod"
                | "const"
                | "static"
                | "return"
                | "crate"
                | "Self"
                | "self"
        ),
        SyntaxLanguage::Python => matches!(
            word,
            "def"
                | "class"
                | "if"
                | "elif"
                | "else"
                | "for"
                | "while"
                | "in"
                | "import"
                | "from"
                | "return"
                | "with"
                | "as"
                | "try"
                | "except"
                | "finally"
                | "lambda"
                | "True"
                | "False"
                | "None"
        ),
        SyntaxLanguage::JavaScript => matches!(
            word,
            "function"
                | "const"
                | "let"
                | "var"
                | "if"
                | "else"
                | "for"
                | "while"
                | "return"
                | "class"
                | "import"
                | "from"
                | "export"
                | "new"
                | "this"
                | "true"
                | "false"
                | "null"
                | "undefined"
        ),
        SyntaxLanguage::PlainText => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_language_by_extension() {
        assert_eq!(detect_language(Some("main.rs")), SyntaxLanguage::Rust);
        assert_eq!(detect_language(Some("app.py")), SyntaxLanguage::Python);
        assert_eq!(detect_language(Some("app.ts")), SyntaxLanguage::JavaScript);
        assert_eq!(
            detect_language(Some("notes.txt")),
            SyntaxLanguage::PlainText
        );
    }

    #[test]
    fn rust_line_tokenization() {
        let line = "let value = 42 // comentario";
        let tokens = tokenize_line(line, SyntaxLanguage::Rust);

        assert_eq!(tokens[0], Some(TokenKind::Keyword));
        assert_eq!(tokens[12], Some(TokenKind::Number));
        assert_eq!(tokens[15], Some(TokenKind::Comment));
    }
}
