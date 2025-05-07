use crate::common::tokens::{TokenKind, fmt_statement_kinds};
use colored::Colorize;
use std::{collections::HashSet, fmt};

fn fmt_loc(line: usize, col: usize, path: &str) -> String {
    let loc = format!("({line}:{col})").yellow().bold();
    let path = format!("in `{path}`").bright_yellow().dimmed();

    format!("{loc} {path}")
}

fn fmt_msg(prefix: &str, msg: String, loc: &Loc, is_tip: bool) -> String {
    let prefix = if is_tip {
        prefix.cyan().bold()
    } else {
        prefix.red().bold()
    };
    let arrow = if is_tip { "⤷".cyan() } else { "⤷".red() };
    let loc = fmt_loc(loc.line, loc.col, &loc.path);

    format!("{prefix} {loc}\n {arrow} {msg}")
}

fn fmt_semantic_err(msg: String, loc: &Loc) -> String {
    fmt_msg("[semantic error]", msg, loc, false)
}

fn fmt_syntax_err(msg: String, loc: &Loc) -> String {
    fmt_msg("[syntax error]", msg, loc, false)
}

fn fmt_semantic_tip(msg: String, loc: &Loc) -> String {
    fmt_msg("[semantic tip]", msg, loc, true)
}

pub fn print_semantic_tip(loc: &Loc, s: String) {
    println!("{}", fmt_semantic_tip(s, loc))
}

pub fn print_err(s: &str) {
    eprintln!("{} {}", "[error]".red().bold(), s);
}

#[derive(Debug)]
pub struct Loc {
    pub path: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug)]
pub enum SyntaxError {
    InvalidChar(Loc, char),
    InvalidDirective(Loc, String),
    InvalidRange(Loc, String),
    InvalidStrEsc(Loc, char),
    InvalidIdent(Loc, String),
    InvalidFloat(Loc, String),
    InvalidFloatExp(&'static str, Loc, String),
    UnterminatedStr(Loc, String),
    UnexpectedToken(Loc, String),
    NoToken(Loc, String),
}

#[derive(Debug)]
pub enum SemanticError {
    ExtraShortName(Loc, HashSet<String>),
    DuplicateName(Loc, HashSet<String>),
    CannotNest(Loc, String),
    EmptyStatement(Loc),
    DiffOptionTypes(Loc, String),
}

#[derive(Debug)]
pub enum ParseError {
    Syntax(SyntaxError),
    Semantic(SemanticError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Semantic(err) => write!(f, "{err}"),
            Self::Syntax(err) => write!(f, "{err}"),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ExtraShortName(loc, hs) => {
                let colored_strs: Vec<String> = hs.iter().map(|s| s.blue().to_string()).collect();
                let joined_msg = colored_strs.join("/");
                let msg = format!(
                    "Argument can only have one short name (got {}): {}",
                    hs.len(),
                    joined_msg
                );
                let fmted = fmt_semantic_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::DuplicateName(loc, hs) => {
                let colored_str: Vec<String> = hs.iter().map(|s| s.blue().to_string()).collect();
                let joined_msg = colored_str.join("/");
                let msg = format!(
                    "Duplicate name(s) found ({}) for argument: {}",
                    hs.len(),
                    joined_msg
                );
                let fmted = fmt_semantic_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::CannotNest(loc, s) => {
                let msg = format!(
                    "{} blocks cannot contain nested blocks (only {})",
                    s.bright_magenta(),
                    "command".bright_magenta()
                );

                let fmted = fmt_semantic_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::EmptyStatement(loc) => {
                let msg = format!(
                    "Statement only contains commas; must include one or more of the following:{}",
                    fmt_statement_kinds()
                );
                let fmted = fmt_semantic_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::DiffOptionTypes(loc, s) => {
                let msg = format!("Option values differ: {s}");
                let fmted = fmt_semantic_err(msg, loc);
                write!(f, "{fmted}")
            }
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidChar(loc, ch) => {
                let msg = format!("Invalid character: {}", format!("'{ch}'").red());
                let fmted = fmt_syntax_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::InvalidDirective(loc, s) => {
                let msg = format!("Invalid directive: {}", s.blue());
                let fmted = fmt_syntax_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::InvalidRange(loc, s) => {
                let msg = format!("Invalid range syntax: {}", s.yellow());
                let fmted = fmt_syntax_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::InvalidStrEsc(loc, ch) => {
                let msg = format!(
                    "Invalid string escape sequence: {}",
                    format!("'\\{ch}'").red()
                );
                let fmted = fmt_syntax_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::InvalidIdent(loc, s) => {
                let msg = format!("Identifier cannot start or end with '-': {}", s.blue());
                let fmted = fmt_syntax_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::InvalidFloat(loc, s) => {
                let msg = format!(
                    "Float literal cannot contain > 1 decimal point: {}",
                    s.yellow()
                );
                let fmted = fmt_syntax_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::InvalidFloatExp(msg, loc, s) => {
                let msg = format!("{msg}: {}", s.yellow());
                let fmted = fmt_syntax_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::UnterminatedStr(loc, s) => {
                let msg = format!("Unterminated string literal: {}", format!("\"{s}").green());
                let fmted = fmt_syntax_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::UnexpectedToken(loc, s) => {
                let fmted = fmt_syntax_err(s.into(), loc);
                write!(f, "{fmted}")
            }

            Self::NoToken(loc, s) => {
                let fmted = fmt_syntax_err(s.into(), loc);
                write!(f, "{fmted}")
            }
        }
    }
}
