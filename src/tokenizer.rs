use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

fn is_path(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    // Linux/unix
    if s.starts_with('/') || s.starts_with("~") {
        return true;
    }

    // Windows
    if s.len() > 2 {
        let bytes = s.as_bytes();
        if bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'\\' || bytes[2] == b'/')
        {
            return true;
        }
    }

    // General fallback
    s.contains('/') || s.contains('\\')
}

#[derive(Debug)]
pub enum Symbol {
    Eq,
    OpenBrace,
    CloseBrace,
    Alternative,
    Range,
}

impl FromStr for Symbol {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "=" => Ok(Self::Eq),
            "{" => Ok(Self::OpenBrace),
            "}" => Ok(Self::CloseBrace),
            "/" => Ok(Self::Alternative),
            ".." => Ok(Self::Range),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum Keyword {
    Command,
    Pos,
    Named,
    Flag,
}

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "command" => Ok(Self::Command),
            "pos" => Ok(Self::Pos),
            "named" => Ok(Self::Named),
            "flag" => Ok(Self::Flag),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum Type {
    String,
    Int,
    Float,
    Path,
    Boolean,
}

impl FromStr for Type {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "string" => Ok(Self::String),
            "int" => Ok(Self::Int),
            "float" => Ok(Self::Float),
            "path" => Ok(Self::Path),
            "bool" => Ok(Self::Boolean),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum Literal {
    String(String),
    Float(f64),
    Int(i64),
    Boolean(bool),
    Path(PathBuf),
}

impl Literal {
    fn from_str(s: &str) -> Result<Self, ()> {
        let s = s.trim();

        // Check for bool first
        match s.to_ascii_lowercase().as_str() {
            "true" => return Ok(Self::Boolean(true)),
            "false" => return Ok(Self::Boolean(false)),
            _ => {}
        }

        if let Ok(i) = s.parse::<i64>() {
            return Ok(Self::Int(i));
        }

        if let Ok(f) = s.parse::<f64>() {
            return Ok(Self::Float(f));
        }

        if s.starts_with("\"") && s.ends_with("\"") {
            let inner_str = &s[1..s.len() - 1];

            if is_path(s) {
                return Ok(Self::Path(PathBuf::from(inner_str)));
            }

            return Ok(Self::String(inner_str.to_string()));
        }

        Err(())
    }
}

#[derive(Debug)]
pub enum Expr {
    Identifier(String),
    Wildcard,
    Literal(Literal),
    Unknown(String),
}

impl Expr {
    pub fn from_str(s: &str) -> Self {
        let s = s.trim();

        if s.to_ascii_lowercase().as_str() == "*" {
            return Self::Wildcard;
        }

        if let Ok(l) = Literal::from_str(s) {
            return Self::Literal(l);
        }

        if s.chars()
            .all(|c| c.is_ascii_alphabetic() || c == '-' || c == '_')
        {
            return Self::Identifier(s.to_string());
        }

        Self::Unknown(s.to_string())
    }
}

lazy_static::lazy_static! {
    static ref RE_ALTERNATIVES: Regex = Regex::new(r"(\w)/(\w)").unwrap();
    static ref RE_RANGES: Regex = Regex::new(r"(\d)..(\d)").unwrap();
}

fn process_content(content: &str) -> Vec<String> {
    let mut content_vec = Vec::new();

    for line in content.lines() {
        let line = line.split('#').next().unwrap().trim();

        if line.is_empty() {
            continue;
        }

        let line = RE_ALTERNATIVES.replace_all(line, "$1 / $2");
        let line = RE_RANGES.replace_all(&line, "$1 .. $2");

        let mut wrapped = false;
        let mut start = 0;
        let line = line.as_ref(); // No clone unless necessary

        for (i, c) in line.char_indices() {
            match c {
                '"' => wrapped = !wrapped,
                ' ' if !wrapped => {
                    if start != i {
                        content_vec.push(line[start..i].to_string());
                    }
                    start = i + 1;
                }
                _ => {}
            }
        }
        if start < line.len() {
            content_vec.push(line[start..].to_string());
        }
    }

    content_vec
}

#[derive(Debug)]
pub enum Token {
    Symbol(Symbol),
    Keyword(Keyword),
    Type(Type),
    Expr(Expr),
}

pub fn tokenize(path: &str) -> Result<Vec<Token>, String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Err("Could not read file".to_string()),
    };

    let mut tokens: Vec<Token> = Vec::new();

    // Step 2: Lex
    for tok in process_content(&content) {
        if let Ok(symbol) = Symbol::from_str(&tok) {
            tokens.push(Token::Symbol(symbol));
            continue;
        }

        if let Ok(kw) = Keyword::from_str(&tok) {
            tokens.push(Token::Keyword(kw));
            continue;
        }

        if let Ok(typ) = Type::from_str(&tok) {
            tokens.push(Token::Type(typ));
            continue;
        }

        let expr = Expr::from_str(&tok);
        tokens.push(Token::Expr(expr));
    }

    Ok(tokens)
}
