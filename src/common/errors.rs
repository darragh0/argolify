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
    ExtraDecimalPoint(Loc, String),
    InvalidFloatExpChars(String, Loc, String),
    InvalidNumChars(String, String, Loc, String),
    UnterminatedStr(Loc, String),
    UnexpectedToken(Loc, String),
    NoToken(Loc, String),
    NoFloatExp(Loc, String),
    InvalidNumberSign(Loc, String),
    InvalidStandaloneSymbol(Loc, String),
}

#[derive(Debug)]
pub enum SemanticError {
    ExtraShortName(Loc, HashSet<String>),
    DuplicateArgNames(Loc, HashSet<String>),
    CannotNest(Loc, String),
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
                let joined_msg = colored_strs.join(" / ");
                let msg = format!(
                    "Argument can only have one short name: {} {} {}",
                    hs.len(),
                    "->".bold(),
                    joined_msg
                );
                let fmted = fmt_semantic_err(msg, loc);
                write!(f, "{fmted}")
            }

            Self::DuplicateArgNames(loc, hs) => {
                let colored_str: Vec<String> = hs.iter().map(|s| s.blue().to_string()).collect();
                let joined_msg = colored_str.join(" / ");
                let msg = format!(
                    "Duplicate name(s) found ({}) for argument {} {}",
                    hs.len(),
                    "->".bold(),
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
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (msg, loc) = match self {
            Self::InvalidChar(loc, ch) => (
                format!("Illegal character {} {}", "->".bold(), ch.to_string().red()),
                loc,
            ),

            Self::InvalidDirective(loc, s) => (
                format!("Invalid directive {} {}", "->".bold(), s.blue()),
                loc,
            ),

            Self::InvalidRange(loc, s) => (
                format!("Invalid range syntax {} {}", "->".bold(), s.yellow()),
                loc,
            ),

            Self::InvalidStrEsc(loc, ch) => (
                format!(
                    "Invalid string escape sequence {} {}",
                    "->".bold(),
                    format!("\\{ch}").red()
                ),
                loc,
            ),

            Self::InvalidIdent(loc, s) => (
                format!(
                    "Identifier cannot start or end with {} {} {}",
                    "-".red(),
                    "->".bold(),
                    s.blue()
                ),
                loc,
            ),

            Self::ExtraDecimalPoint(loc, s) => (
                format!(
                    "Float literal cannot contain > 1 decimal point {} {}",
                    "->".bold(),
                    s.yellow()
                ),
                loc,
            ),

            Self::NoFloatExp(loc, s) => (
                format!("Missing float exponent {} {}", "->".bold(), s.yellow()),
                loc,
            ),

            Self::InvalidFloatExpChars(chars, loc, s) => (
                format!(
                    "Float exponent contains illegal character(s): {} {} {}",
                    chars
                        .chars()
                        .map(|c| c.to_string().bright_red().to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    "->".bold(),
                    s.yellow(),
                ),
                loc,
            ),

            Self::InvalidNumChars(float_or_int, chars, loc, s) => (
                format!(
                    "{} contains illegal character(s): {} {} {}",
                    float_or_int,
                    chars
                        .chars()
                        .map(|c| c.to_string().bright_red().to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    "->".bold(),
                    s.yellow()
                ),
                loc,
            ),

            Self::UnterminatedStr(loc, s) => (
                format!(
                    "Unterminated string literal {} {}",
                    "->".bold(),
                    format!("\"{s}").green()
                ),
                loc,
            ),

            Self::UnexpectedToken(loc, s) => (s.into(), loc),

            Self::NoToken(loc, s) => (s.into(), loc),

            Self::InvalidNumberSign(loc, s) => (
                format!("Multiple signs found {} {}", "->".bold(), s.yellow()),
                loc,
            ),

            Self::InvalidStandaloneSymbol(loc, s) => (
                format!(
                    "Invalid standalone symbol(s) {} {}",
                    "->".bold(),
                    s.yellow()
                ),
                loc,
            ),
        };

        write!(f, "{}", fmt_syntax_err(msg, loc))
    }
}
