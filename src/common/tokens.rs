use colored::Colorize;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    ExclusiveRange,
    InclusiveRange,
    Minus,
    Plus,
    SemiColon,
    Action,
    Int,
    Directive,
    Float,
    Comma,
    OpenBrace,
    CloseBrace,
    Alt,
    Eq,
    Keyword,
    Boolean,
    Wildcard,
    Type,
    String,
    Identifier,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TokenKind {
    fn as_str(&self) -> String {
        match self {
            TokenKind::ExclusiveRange => format!("exclusive range {}", "..".yellow()),
            TokenKind::InclusiveRange => format!("inclusive range {}", "..=".yellow()),
            TokenKind::Minus => format!("minus {}", "-".yellow()),
            TokenKind::Plus => format!("plus {}", "+".yellow()),
            TokenKind::SemiColon => "semicolon ;".to_string(),
            TokenKind::Action => "action".to_string(),
            TokenKind::Int => "integer".to_string(),
            TokenKind::Directive => "directive".to_string(),
            TokenKind::Float => "float".to_string(),
            TokenKind::Comma => "comma ,".to_string(),
            TokenKind::OpenBrace => "open brace {".to_string(),
            TokenKind::CloseBrace => "close brace }".to_string(),
            TokenKind::Alt => "alternative /".to_string(),
            TokenKind::Eq => format!("equal sign {}", "=".yellow()),
            TokenKind::Keyword => "keyword".to_string(),
            TokenKind::Boolean => format!("{}/{}", "true".bright_red(), "false".bright_red()),
            TokenKind::Wildcard => format!("wildcard {}", "*".yellow()),
            TokenKind::Type => "type".to_string(),
            TokenKind::String => "string".to_string(),
            TokenKind::Identifier => "identifer".to_string(),
        }
    }

    pub fn help(&self) -> String {
        match self {
            TokenKind::ExclusiveRange => format!("{:20} {}", "exclusive range:", "..".yellow()),
            TokenKind::InclusiveRange => format!("{:20} {}", "inclusive range:", "..=".yellow()),
            TokenKind::Minus => format!("{:20} {}", "minus:", "-".yellow()),
            TokenKind::Plus => format!("{:20} {}", "plus:", "+".yellow()),
            TokenKind::SemiColon => format!("{:20} {}", "semicolon:", ";"),
            TokenKind::Action => format!(
                "{:20} {} / {}",
                "action:",
                "show_help".bright_magenta(),
                "show_version".bright_magenta()
            ),
            TokenKind::Int => format!("{:20} {}", "integer:", "[+/-]<1-9>[0-9 ...]".yellow()),
            TokenKind::Directive => format!(
                "{:20} {}/{}",
                "directive:",
                "!version".blue(),
                "!program".blue()
            ),
            TokenKind::Float => format!(
                "{:20} {}",
                "float:",
                "[+/-][0-9 ...].[0-9 ...][e<[+/-]<integer>]".yellow()
            ),
            TokenKind::Comma => format!("{:20} {}", "comma:", ","),
            TokenKind::OpenBrace => format!("{:20} {}", "open brace:", "{"),
            TokenKind::CloseBrace => format!("{:20} {}", "close brace:", "}"),
            TokenKind::Alt => format!("{:20} {}", "alternative:", "/"),
            TokenKind::Eq => format!("{:20} {}", "equal sign:", "="),
            TokenKind::Keyword => format!(
                "{:20} {} / {} / {} / {}",
                "keyword:",
                "command".bright_magenta(),
                "pos".bright_magenta(),
                "named".bright_magenta(),
                "flag".bright_magenta()
            ),
            TokenKind::Boolean => format!(
                "{:20} {} / {}",
                "boolean:",
                "true".bright_red(),
                "false".bright_red()
            ),
            TokenKind::Wildcard => format!("{:20} {}", "wildcard:", "*".yellow()),
            TokenKind::Type => format!(
                "{:20} {} / {} / {} / {} / {}",
                "type:",
                "int".bright_cyan(),
                "float".bright_cyan(),
                "uint".bright_cyan(),
                "path".bright_cyan(),
                "bool".bright_cyan()
            ),
            TokenKind::String => format!("{:20} {}", "string:", "\"\"".green()),
            TokenKind::Identifier => format!("{:20} {}", "identifier:", ""),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fmted = match self.kind {
            TokenKind::String => format!("\"{}\"", self.lexeme).green(),
            TokenKind::Identifier | TokenKind::Directive => self.lexeme.blue(),
            TokenKind::Float
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::ExclusiveRange
            | TokenKind::InclusiveRange
            | TokenKind::Eq
            | TokenKind::Wildcard
            | TokenKind::Int => self.lexeme.yellow(),
            TokenKind::Type => self.lexeme.bright_cyan(),
            TokenKind::Boolean => self.lexeme.bright_red(),
            TokenKind::Keyword | TokenKind::Action => self.lexeme.bright_magenta(),
            _ => self.lexeme.white(),
        };
        write!(f, "{fmted}")
    }
}

impl Token {
    pub fn len(&self) -> usize {
        match self.kind {
            TokenKind::String => self.lexeme.len() + 2,
            _ => self.lexeme.len(),
        }
    }
}

pub const ASSIGNMENT_KINDS: [TokenKind; 9] = [
    TokenKind::String,
    TokenKind::Boolean,
    TokenKind::Float,
    TokenKind::Int,
    TokenKind::ExclusiveRange,
    TokenKind::InclusiveRange,
    TokenKind::Wildcard,
    TokenKind::Type,
    TokenKind::Action,
];

pub fn fmt_assignment_kinds() -> String {
    ASSIGNMENT_KINDS
        .iter()
        .filter_map(|s| match s {
            TokenKind::SemiColon | TokenKind::Comma => None,
            _ => Some(format!("\n     - {}", s.help())),
        })
        .collect::<String>()
}

pub fn is_assignment_kind(kind: &TokenKind) -> bool {
    ASSIGNMENT_KINDS.contains(kind)
}
