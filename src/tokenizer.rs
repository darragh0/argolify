use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;

use crate::common::errors::{Loc, ParseError, ParseError::*, SyntaxError};
use crate::common::tokens::{Token, TokenKind};

fn is_special_symbol(symbol: char) -> bool {
    ";={}/,-+*#".contains(symbol)
}

fn is_num_tok(c: char) -> bool {
    c.is_ascii_digit() || c == '_'
}

pub fn tokenize(content: &str, path: &Path) -> Result<Vec<Token>, ParseError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut col = 0;
    let mut line = 1;
    let mut chars = content.chars().peekable();
    let path = path.to_string_lossy().to_string();

    while let Some(&ch) = chars.peek() {
        match ch {
            '#' | '\t' | '\n' | ' ' => skip_tokens(&mut chars, &mut line, &mut col),

            ';' | '=' | '{' | '}' | '/' | ',' | '-' | '+' | '*' => {
                let tok = parse_symbol(&mut chars, ch, &mut line, &mut col);
                tokens.push(tok);
            }

            '"' => {
                let tok = parse_str(&mut chars, &mut line, &mut col, &path)?;
                tokens.push(tok);
            }

            'a'..='z' | 'A'..='Z' | '_' | '!' => {
                let tok = parse_ident(&mut chars, &mut line, &mut col, &path)?;
                tokens.push(tok);
            }

            '.' | '0'..='9' => {
                let mut lexeme: String = ch.into();
                let mut is_range = false;
                let mut cloned = chars.clone();

                cloned.next();
                let start_col = col;

                if let Some('.') = cloned.peek() {
                    cloned.next();
                    col += 2;
                    lexeme.push('.');
                    is_range = true;
                }

                if let Some('=') = cloned.peek() {
                    cloned.next();
                    col += 1;
                    lexeme.push('=');

                    if !is_range {
                        let loc = Loc { path, line, col };
                        return Err(Syntax(SyntaxError::InvalidRange(loc, lexeme)));
                    }

                    chars.next();
                    chars.next();
                    chars.next();
                    tokens.push(Token {
                        kind: TokenKind::InclusiveRange,
                        lexeme,
                        col: start_col,
                        line,
                    });
                    continue;
                }

                if is_range {
                    chars.next();
                    chars.next();
                    tokens.push(Token {
                        kind: TokenKind::ExclusiveRange,
                        lexeme,
                        col: start_col,
                        line,
                    });
                    continue;
                }

                let tok = parse_number(&mut chars, &mut line, &mut col, &path)?;
                tokens.push(tok);
            }

            _ => {
                let loc = Loc { path, line, col };
                return Err(Syntax(SyntaxError::InvalidChar(loc, ch)));
            }
        }
    }

    Ok(tokens)
}

fn skip_tokens(chars: &mut Peekable<Chars>, line: &mut usize, col: &mut usize) {
    if let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                chars.next();
                *col += 1;
            }
            '\n' => {
                chars.next();
                *col = 0;
                *line += 1;
            }
            '#' => {
                chars.next();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch != '\n' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

fn parse_symbol(chars: &mut Peekable<Chars>, ch: char, line: &mut usize, col: &mut usize) -> Token {
    match ch {
        ';' | '=' | '{' | '}' | '/' | ',' | '-' | '+' | '*' => {
            chars.next();
            *col += 1;
            let kind = match ch {
                ';' => TokenKind::SemiColon,
                '=' => TokenKind::Eq,
                '{' => TokenKind::OpenBrace,
                '}' => TokenKind::CloseBrace,
                '/' => TokenKind::Alt,
                ',' => TokenKind::Comma,
                '-' => TokenKind::Minus,
                '+' => TokenKind::Plus,
                '*' => TokenKind::Wildcard,
                _ => unreachable!(),
            };
            Token {
                kind,
                lexeme: ch.to_string(),
                line: *line,
                col: *col,
            }
        }
        _ => unreachable!(),
    }
}

fn parse_str(
    chars: &mut Peekable<Chars>,
    line: &mut usize,
    col: &mut usize,
    path: &String,
) -> Result<Token, ParseError> {
    chars.next(); // consume "
    *col += 1;

    let start_col = *col;
    let mut lexeme = String::new();
    let mut in_esc = false;

    while let Some(&next_ch) = chars.peek() {
        *col += 1;
        chars.next();
        if next_ch == '"' {
            break;
        }

        if next_ch == '\n' {
            *col -= 1;
            let loc = Loc {
                path: path.to_string(),
                line: *line,
                col: *col,
            };
            return Err(Syntax(SyntaxError::UnterminatedStr(loc, lexeme)));
        }

        if next_ch == '\\' {
            in_esc = true;
        } else if in_esc {
            let esc_ch = match next_ch {
                '"' => '"',
                '\\' => '\\',
                't' => '\t',
                'b' => '\u{08}',
                'e' => '\u{1B}',
                _ => {
                    *col -= 1;
                    let loc = Loc {
                        path: path.to_string(),
                        line: *line,
                        col: *col,
                    };
                    return Err(Syntax(SyntaxError::InvalidStrEsc(loc, next_ch)));
                }
            };
            lexeme.push(esc_ch);
            in_esc = false;
        } else {
            lexeme.push(next_ch);
        }
    }

    Ok(Token {
        kind: TokenKind::String,
        lexeme,
        line: *line,
        col: start_col,
    })
}

fn parse_ident(
    chars: &mut Peekable<Chars>,
    line: &mut usize,
    col: &mut usize,
    path: &String,
) -> Result<Token, ParseError> {
    let mut lexeme = String::new();
    let start_col = *col + 1;
    let mut is_directive = false;

    if let Some('!') = chars.peek() {
        is_directive = true;
        *col += 1;
        lexeme.push('!');
        chars.next(); // consume !
    }

    while let Some(&next_ch) = chars.peek() {
        if next_ch.is_ascii_alphanumeric() || "-_".contains(next_ch) {
            lexeme.push(next_ch);
            *col += 1;
            chars.next();
        } else {
            break;
        }
    }

    let ident = if is_directive { &lexeme[1..] } else { &lexeme };
    if ident.starts_with('-') || ident.ends_with('-') {
        let loc = Loc {
            path: path.to_string(),
            line: *line,
            col: *col,
        };
        return Err(Syntax(SyntaxError::InvalidIdent(loc, lexeme)));
    }

    let token_kind = match lexeme.as_str() {
        "!version" | "!program" => TokenKind::Directive,
        "show_version" | "show_help" => TokenKind::Action,
        "true" | "false" => TokenKind::Boolean,
        "int" | "float" | "uint" | "path" | "bool" => TokenKind::Type,
        "command" | "flag" | "pos" | "named" => TokenKind::Keyword,
        _ => {
            if is_directive {
                let loc = Loc {
                    path: path.to_string(),
                    line: *line,
                    col: *col,
                };
                return Err(Syntax(SyntaxError::InvalidDirective(loc, lexeme)));
            }
            TokenKind::Identifier
        }
    };

    Ok(Token {
        kind: token_kind,
        lexeme,
        line: *line,
        col: start_col,
    })
}

fn parse_number(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    line: &mut usize,
    col: &mut usize,
    path: &String,
) -> Result<Token, ParseError> {
    let mut lexeme = String::new();
    let start_col = *col + 1;
    let mut is_float = false;
    let mut err_dp = 0;

    while let Some(&d) = chars.peek() {
        if is_num_tok(d) {
            *col += 1;
            lexeme.push(d);
            chars.next();
        } else if d == '.' {
            if is_float {
                err_dp = *col + 1;
            }
            is_float = true;
            *col += 1;
            lexeme.push('.');
            chars.next();
        } else {
            break;
        }
    }

    let mut has_exp = false;
    if let Some(&e) = chars.peek() {
        if e.eq_ignore_ascii_case(&'e') {
            is_float = true;
            lexeme.push(e);
            chars.next();
            *col += 1;

            let mut unrec_c = String::new();
            while let Some(&n) = chars.peek() {
                if is_num_tok(n) {
                    has_exp = true;
                    lexeme.push(n);
                    chars.next();
                    *col += 1;
                } else if "+-".contains(n) {
                    lexeme.push(n);
                    chars.next();
                    *col += 1;
                } else if is_special_symbol(n) || n.is_whitespace() {
                    break;
                } else {
                    unrec_c.push(n);
                    lexeme.push(n);
                    chars.next();
                    *col += 1;
                }
            }

            if err_dp != 0 {
                let loc = Loc {
                    path: path.to_string(),
                    line: *line,
                    col: err_dp,
                };

                return Err(Syntax(SyntaxError::InvalidFloat(loc, lexeme)));
            }

            if !has_exp {
                let msg = "Missing float exponent";
                let loc = Loc {
                    path: path.to_string(),
                    line: *line,
                    col: err_dp,
                };
                return Err(Syntax(SyntaxError::InvalidFloatExp(msg, loc, lexeme)));
            }

            if !unrec_c.is_empty() {
                let msg = "Float exponent contains illegal character(s)";
                let loc = Loc {
                    path: path.to_string(),
                    line: *line,
                    col: *col,
                };

                return Err(Syntax(SyntaxError::InvalidFloatExp(msg, loc, lexeme)));
            }
        }
    }

    if err_dp != 0 {
        let loc = Loc {
            path: path.to_string(),
            line: *line,
            col: err_dp,
        };

        return Err(Syntax(SyntaxError::InvalidFloat(loc, lexeme)));
    }

    Ok(Token {
        kind: if is_float {
            TokenKind::Float
        } else {
            TokenKind::Int
        },
        lexeme,
        line: *line,
        col: start_col,
    })
}
