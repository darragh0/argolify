use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;

use crate::common::errors::{Loc, ParseError, ParseError::*, SyntaxError};
use crate::common::tokens::{Token, TokenKind};
use crate::common::util::CharExtensions;

pub fn tokenize(content: &str, path: &Path) -> Result<Vec<Token>, ParseError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut col = 0;
    let mut line = 1;
    let mut chars = content.chars().peekable();
    let path = path.to_string_lossy().to_string();

    while let Some(&ch) = chars.peek() {
        match ch {
            '#' | '\t' | '\n' | ' ' => skip_tokens(&mut chars, &mut line, &mut col),

            ';' | '=' | '{' | '}' | '/' | ',' | '*' => {
                let tok = parse_symbol(&mut chars, ch, &mut line, &mut col);
                tokens.push(tok);
            }

            'a'..='z' | 'A'..='Z' | '_' | '!' => {
                let tok = parse_ident(&mut chars, &mut line, &mut col, &path)?;
                tokens.push(tok);
            }

            '-' | '+' | '.' | '0'..='9' => {
                let tok = parse_range_or_number(&mut chars, &mut line, &mut col, &path)?;
                tokens.push(tok);
            }

            '"' => {
                let tok = parse_str(&mut chars, &mut line, &mut col, &path)?;
                tokens.push(tok);
            }

            _ => {
                col += 1;
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
        if next_ch == '"' && !in_esc {
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

    let ident = if is_directive { &lexeme } else { &lexeme };
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

fn parse_range_or_number(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    line: &mut usize,
    col: &mut usize,
    path: &String,
) -> Result<Token, ParseError> {
    let start_col = *col + 1;

    let res = parse_number_inner(chars, line, col, path);
    let (first, fallback_err) = match res {
        Ok(tok) => (tok, None),
        Err(Syntax(SyntaxError::InvalidStandaloneSymbol(loc, s))) => (
            Token {
                kind: TokenKind::Int,
                lexeme: "".into(),
                line: 0,
                col: 0,
            },
            Some(Syntax(SyntaxError::InvalidStandaloneSymbol(loc, s))),
        ),
        Err(e) => return Err(e),
    };

    // Peek for '..' or '..='
    if let Some(&'.') = chars.peek() {
        let mut temp_iter = chars.clone();
        temp_iter.next(); // first dot
        if let Some('.') = temp_iter.next() {
            // matched '..'
            *col += 2;
            chars.next();
            chars.next();

            let inclusive = if let Some(&'=') = chars.peek() {
                *col += 1;
                chars.next();
                true
            } else {
                false
            };

            let second = match parse_number_inner(chars, line, col, path) {
                Err(Syntax(SyntaxError::InvalidStandaloneSymbol(_, _))) => Token {
                    kind: TokenKind::Int,
                    lexeme: "".into(),
                    line: 0,
                    col: 0,
                },
                Ok(tok) => tok,
                Err(e) => return Err(e),
            };
            let combined_lexeme = format!(
                "{}..{}{}",
                first.lexeme,
                if inclusive { "=" } else { "" },
                second.lexeme
            );

            return Ok(Token {
                kind: if inclusive {
                    TokenKind::InclusiveRange
                } else {
                    TokenKind::ExclusiveRange
                },
                lexeme: combined_lexeme,
                line: *line,
                col: start_col,
            });
        }
    }

    if let Some(err) = fallback_err {
        Err(err)
    } else {
        Ok(first)
    }
}

fn parse_number_inner(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    line: &mut usize,
    col: &mut usize,
    path: &String,
) -> Result<Token, ParseError> {
    let mut lexeme = String::new();
    let start_col = *col + 1;
    let mut is_float = false;
    let mut err_dp = 0;
    let mut digit_found = false;
    let mut invalid_range = false;

    // Grab all leading +/- symbols
    while let Some(&c) = chars.peek() {
        if c.is_num_sign() {
            lexeme.push(c);
            chars.next();
            *col += 1;
        } else {
            break;
        }
    }

    // Only 0 or 1 sign is allowed
    if lexeme.chars().filter(|c| "+-".contains(*c)).count() > 1 {
        let loc = Loc {
            path: path.clone(),
            line: *line,
            col: start_col,
        };
        return Err(Syntax(SyntaxError::InvalidNumberSign(loc, lexeme.clone())));
    }

    let mut unrec_c = String::new();
    while let Some(&d) = chars.peek() {
        if d.is_num_char() {
            digit_found = true;
            lexeme.push(d);
            chars.next();
            *col += 1;
        } else if d == '.' {
            // Look ahead for `..` — if so, stop parsing
            let mut temp = chars.clone();
            temp.next(); // skip current '.'
            if let Some(&next) = temp.peek() {
                if next == '.' && is_float {
                    invalid_range = true;
                } else if next == '.' {
                    temp.next();
                    if let Some(&nextnext) = temp.peek() {
                        if nextnext == '.' {
                            invalid_range = true;
                        } else {
                            break;
                        }
                    }
                }
            }

            if is_float {
                err_dp = *col + 1;
            }

            is_float = true;
            lexeme.push('.');
            chars.next();
            *col += 1;
        } else if d.is_other_char() {
            break;
        } else {
            unrec_c.push(d);
            lexeme.push(d);
            chars.next();
            *col += 1;
        }
    }

    if !unrec_c.is_empty() {
        return Err(Syntax(SyntaxError::InvalidNumChars(
            (if is_float { "Float" } else { "Integer" }).into(),
            unrec_c,
            Loc {
                path: path.clone(),
                line: *line,
                col: *col,
            },
            lexeme.clone(),
        )));
    }

    if invalid_range {
        return Err(Syntax(SyntaxError::InvalidRange(
            Loc {
                path: path.clone(),
                line: *line,
                col: start_col,
            },
            lexeme.clone(),
        )));
    }

    // Validate that something numeric came after sign
    if !digit_found && !is_float {
        let loc = Loc {
            path: path.clone(),
            line: *line,
            col: start_col,
        };
        return Err(Syntax(SyntaxError::InvalidStandaloneSymbol(loc, lexeme)));
    }

    // Disallow "." or "-" or "-." etc as standalone
    if lexeme == "." || lexeme == "-" || lexeme == "+" || lexeme == "-." || lexeme == "+." {
        let loc = Loc {
            path: path.clone(),
            line: *line,
            col: start_col,
        };
        return Err(Syntax(SyntaxError::InvalidStandaloneSymbol(loc, lexeme)));
    }

    // Handle optional float exponent
    if let Some(&e) = chars.peek() {
        if e.eq_ignore_ascii_case(&'e') {
            is_float = true;
            lexeme.push(e);
            chars.next();
            *col += 1;

            // Optional single sign after e
            if let Some(&sign) = chars.peek() {
                if sign.is_num_sign() {
                    lexeme.push(sign);
                    chars.next();
                    *col += 1;

                    // Disallow multiple signs
                    if let Some(&next) = chars.peek() {
                        if "+-".contains(next) {
                            let loc = Loc {
                                path: path.clone(),
                                line: *line,
                                col: *col + 1,
                            };
                            return Err(Syntax(SyntaxError::InvalidNumberSign(loc, lexeme)));
                        }
                    }
                }
            }

            let mut digit_in_exp = false;
            let mut unrec_c = String::new();

            while let Some(&n) = chars.peek() {
                if n.is_num_char() {
                    digit_in_exp = true;
                    lexeme.push(n);
                    chars.next();
                    *col += 1;
                } else if n.is_other_char() {
                    break;
                } else {
                    unrec_c.push(n);
                    lexeme.push(n);
                    chars.next();
                    *col += 1;
                }
            }

            if !digit_in_exp {
                return Err(Syntax(SyntaxError::NoFloatExp(
                    Loc {
                        path: path.clone(),
                        line: *line,
                        col: *col,
                    },
                    lexeme.clone(),
                )));
            }

            if !unrec_c.is_empty() {
                return Err(Syntax(SyntaxError::InvalidFloatExpChars(
                    unrec_c,
                    Loc {
                        path: path.clone(),
                        line: *line,
                        col: *col,
                    },
                    lexeme.clone(),
                )));
            }
        }
    }

    if err_dp != 0 {
        return Err(Syntax(SyntaxError::ExtraDecimalPoint(
            Loc {
                path: path.clone(),
                line: *line,
                col: err_dp,
            },
            lexeme.clone(),
        )));
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
