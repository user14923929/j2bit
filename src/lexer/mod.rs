use anyhow::{bail, Result};

/// Токен с позицией в исходнике (для сообщений об ошибках)
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Литералы
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),

    // Идентификаторы и ключевые слова
    Ident(String),
    Class,
    Void,
    Return,
    If,
    Else,
    While,
    For,
    New,
    Static,
    Public,
    Private,
    Final,
    Int,
    Float,
    Bool,
    StringType,

    // Операторы
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,    // =
    EqEq,  // ==
    NotEq, // !=
    Lt,    // <
    LtEq,  // <=
    Gt,    // >
    GtEq,  // >=
    And,   // &&
    Or,    // ||
    Not,   // !
    Dot,   // .
    Comma,
    Semicolon,
    Colon,

    // Скобки
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Eof,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    let mut line = 1;
    let mut col = 1;

    while i < chars.len() {
        let start_col = col;
        let c = chars[i];

        // Пропускаем пробелы
        if c == '\n' {
            line += 1;
            col = 1;
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            col += 1;
            i += 1;
            continue;
        }

        // Однострочный комментарий
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Многострочный комментарий
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            col += 2;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    col += 2;
                    break;
                }
                if chars[i] == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                i += 1;
            }
            continue;
        }

        // Числа
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
                col += 1;
            }
            if i < chars.len() && chars[i] == '.' {
                i += 1;
                col += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                    col += 1;
                }
                let num: f64 = chars[start..i].iter().collect::<String>().parse()?;
                tokens.push(Token {
                    kind: TokenKind::FloatLiteral(num),
                    line,
                    col: start_col,
                });
            } else {
                let num: i64 = chars[start..i].iter().collect::<String>().parse()?;
                tokens.push(Token {
                    kind: TokenKind::IntLiteral(num),
                    line,
                    col: start_col,
                });
            }
            continue;
        }

        // Строки
        if c == '"' {
            i += 1;
            col += 1;
            let start = i;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\n' {
                    bail!("unterminated string at line {line}");
                }
                i += 1;
                col += 1;
            }
            let s: String = chars[start..i].iter().collect();
            i += 1; // закрывающая "
            col += 1;
            tokens.push(Token {
                kind: TokenKind::StringLiteral(s),
                line,
                col: start_col,
            });
            continue;
        }

        // Идентификаторы и ключевые слова
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
                col += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let kind = match word.as_str() {
                "class" => TokenKind::Class,
                "void" => TokenKind::Void,
                "return" => TokenKind::Return,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "while" => TokenKind::While,
                "for" => TokenKind::For,
                "new" => TokenKind::New,
                "static" => TokenKind::Static,
                "public" => TokenKind::Public,
                "private" => TokenKind::Private,
                "final" => TokenKind::Final,
                "int" => TokenKind::Int,
                "float" => TokenKind::Float,
                "boolean" => TokenKind::Bool,
                "String" => TokenKind::StringType,
                "true" => TokenKind::BoolLiteral(true),
                "false" => TokenKind::BoolLiteral(false),
                _ => TokenKind::Ident(word),
            };
            tokens.push(Token {
                kind,
                line,
                col: start_col,
            });
            continue;
        }

        // Двухсимвольные операторы
        let two = if i + 1 < chars.len() {
            Some((chars[i], chars[i + 1]))
        } else {
            None
        };

        if let Some(kind) = two.and_then(|t| match t {
            ('=', '=') => Some(TokenKind::EqEq),
            ('!', '=') => Some(TokenKind::NotEq),
            ('<', '=') => Some(TokenKind::LtEq),
            ('>', '=') => Some(TokenKind::GtEq),
            ('&', '&') => Some(TokenKind::And),
            ('|', '|') => Some(TokenKind::Or),
            _ => None,
        }) {
            tokens.push(Token {
                kind,
                line,
                col: start_col,
            });
            i += 2;
            col += 2;
            continue;
        }

        // Односимвольные
        let kind = match c {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => TokenKind::Eq,
            '<' => TokenKind::Lt,
            '>' => TokenKind::Gt,
            '!' => TokenKind::Not,
            '.' => TokenKind::Dot,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            other => bail!(
                "unexpected character '{}' at line {line}, col {start_col}",
                other
            ),
        };
        tokens.push(Token {
            kind,
            line,
            col: start_col,
        });
        i += 1;
        col += 1;
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        line,
        col,
    });
    Ok(tokens)
}
