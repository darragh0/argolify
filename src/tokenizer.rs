use crate::common::util::print_syntax_err;
use std::fs;

#[derive(Debug)]
pub enum TokenKind {
    ExclusiveRange,
    InclusiveRange,
    Int,
    Float,
    OpenBrace,
    CloseBrace,
    Alt,
    Eq,
    Keyword,
    Boolean,
    Wildcard,
    String,
    Identifer,
}

fn is_special_symbol(symbol: char) -> bool {
    "+-={}/*".contains(symbol)
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

pub fn tokenize(path: &str) -> Result<Vec<Token>, &'static str> {
    let content = fs::read_to_string(path)
        .map_err(|_| "Could not read file".to_string())
        .unwrap();
    // let content = "command 8.3e-0 i/i{ test = {\"hi there\"} * -58.3 ..* 0.3 hi = \"3\" }";

    let mut tokens: Vec<Token> = Vec::new();
    let mut line = 1;
    let mut col = 1;
    let mut chars = content.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                col += 1;
                chars.next();
            }
            '#' => {
                while let Some(&next_c) = chars.peek() {
                    if next_c == '\n' {
                        line += 1;
                        col = 1;
                        break;
                    }
                    chars.next();
                }
            }
            '\n' => {
                line += 1;
                col = 1;
                chars.next();
            }
            '=' | '{' | '}' | '/' => {
                let kind = match c {
                    '=' => TokenKind::Eq,
                    '{' => TokenKind::OpenBrace,
                    '}' => TokenKind::CloseBrace,
                    '/' => TokenKind::Alt,
                    _ => unreachable!(),
                };
                col += 1;
                chars.next();
                tokens.push(Token {
                    kind,
                    lexeme: c.to_string(),
                    line,
                    col,
                });
            }
            '*' => {
                let mut lookahead = chars.clone();
                lookahead.next();
                if let Some(&next_c) = lookahead.peek() {
                    if next_c == '.' || next_c.is_ascii_digit() {
                        let token = parse_number(&mut chars, &mut line, &mut col, path)?;
                        tokens.push(token);
                    } else {
                        col += 1;
                        chars.next();
                        tokens.push(Token {
                            kind: TokenKind::Wildcard,
                            lexeme: c.to_string(),
                            line,
                            col,
                        });
                    }
                }
            }
            '0'..='9' | '.' | '-' | '+' => {
                let token = parse_number(&mut chars, &mut line, &mut col, path)?;
                tokens.push(token);
            }
            '"' => {
                let mut lexeme = String::new();

                chars.next(); // consume open dquote
                col += 1;

                let mut in_escape = false;
                while let Some(&next_c) = chars.peek() {
                    if next_c == '"' {
                        chars.next(); // consume close dquote
                        col += 1;
                        break;
                    } else if next_c == '\n' {
                        col += 1;
                        print_syntax_err(
                            "unterminated string literal",
                            Some(lexeme),
                            line,
                            col,
                            path,
                        );
                        return Err("");
                    } else if next_c == '\\' {
                        chars.next(); // consume backslash
                        col += 1;
                        in_escape = true;
                    } else if in_escape {
                        let escaped_c = match next_c {
                            '"' => '"',
                            '\\' => '\\',
                            't' => '\t',
                            'b' => '\u{08}',
                            'e' => '\u{1B}',
                            _ => {
                                let err_msg = Some(format!("\\{c}"));
                                print_syntax_err(
                                    "invalid escape sequence",
                                    err_msg,
                                    line,
                                    col,
                                    path,
                                );
                                return Err("");
                            }
                        };
                        col += 1;
                        lexeme.push(escaped_c);
                        chars.next();
                    } else {
                        col += 1;
                        lexeme.push(next_c);
                        chars.next();
                    }
                }

                tokens.push(Token {
                    kind: TokenKind::String,
                    lexeme,
                    line,
                    col,
                });

                chars.next();
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut lexeme = String::new();

                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_alphanumeric() || next_c == '_' || next_c == '-' {
                        lexeme.push(next_c);
                        col += 1;
                        chars.next();
                    } else {
                        break;
                    }
                }

                if lexeme.ends_with('-') {
                    let err_msg = Some(format!("'{lexeme}'"));
                    print_syntax_err("identifer cannot end with '-'", err_msg, line, col, path);
                    return Err("");
                }

                if ["true", "false"].contains(&lexeme.as_str()) {
                    tokens.push(Token {
                        kind: TokenKind::Boolean,
                        lexeme,
                        line,
                        col,
                    });

                    continue;
                }

                let kind = match lexeme.as_str() {
                    "command" => TokenKind::Keyword,
                    "flag" | "pos" | "named" => TokenKind::Keyword,
                    _ => TokenKind::Identifer,
                };

                tokens.push(Token {
                    kind,
                    lexeme,
                    line,
                    col,
                })
            }
            _ => {
                let err_msg = Some(format!("'{c}'"));
                print_syntax_err("invalid char", err_msg, line, col, path);
                return Err("");
            }
        }
    }

    Ok(tokens)
}

fn parse_number(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    line: &mut usize,
    col: &mut usize,
    path: &str,
) -> Result<Token, &'static str> {
    let start_col = *col;
    let start_line = *line;
    let mut lexeme = String::new();
    let mut is_float = false;

    let mut has_digits_before_dot = false;
    let mut has_digits_after_dot = false;

    let mut has_sign = false;

    let mut invalid_range = false;
    let mut range_has_dot = false;
    let mut range_has_eq = false;
    let mut range_has_sign = false;
    let mut range_has_star = false;
    let mut begins_with_star = false;

    while let Some(&sign) = chars.peek() {
        if sign == '+' || sign == '-' {
            has_sign = true;
            lexeme.push(sign);
            chars.next();
            *col += 1;
        } else {
            break;
        }
    }

    if let Some(&'0') = chars.peek() {
        lexeme.push('0');
        chars.next();
        *col += 1;

        if let Some(&prefix) = chars.peek() {
            match prefix {
                'x' | 'o' | 'b' => {
                    let is_valid = match prefix {
                        'x' => |c: char| c.is_ascii_hexdigit(),
                        'o' => |c| ('0'..='7').contains(&c),
                        'b' => |c| ('0'..='1').contains(&c),
                        _ => unreachable!(),
                    };
                    return parse_prefixed_int(
                        chars,
                        col,
                        start_line,
                        start_col,
                        &prefix.to_string(),
                        is_valid,
                        path,
                    );
                }
                '.' => {
                    is_float = true;
                    has_digits_before_dot = true;
                }
                _ => {}
            }
        }
    }

    // Before decimal point
    while let Some(&d) = chars.peek() {
        if d.is_ascii_digit() || d == '_' {
            has_digits_before_dot = true;
            lexeme.push(d);
            chars.next();
            *col += 1;
        } else if d == '*' {
            begins_with_star = true;
            lexeme.push(d);
            chars.next();
            *col += 1;
        } else {
            break;
        }
    }

    // Decimal point
    if let Some(&'.') = chars.peek() {
        is_float = true;
        let insert_pos = lexeme.len();
        lexeme.push('.');
        chars.next();
        *col += 1;

        // After decimal point
        while let Some(&d) = chars.peek() {
            if d.is_ascii_digit() || d == '_' {
                if range_has_star {
                    invalid_range = true;
                }

                has_digits_after_dot = true;
                lexeme.push(d);
                chars.next();
                *col += 1;
            } else if d == '*' {
                if range_has_star || range_has_sign {
                    invalid_range = true;
                }
                range_has_star = true;
                has_digits_after_dot = true;
                lexeme.push('*');
                chars.next();
                *col += 1;
            } else if d == '=' {
                if !range_has_dot || range_has_eq {
                    invalid_range = true;
                }
                range_has_eq = true;
                lexeme.push('=');
                chars.next();
                *col += 1;
            } else if d == '.' {
                if range_has_dot || begins_with_star {
                    invalid_range = true;
                } else if !has_digits_before_dot {
                    // assume start range at 0
                    lexeme.insert(insert_pos, '0');
                }

                range_has_dot = true;
                lexeme.push('.');
                chars.next();
                *col += 1;
            } else if range_has_dot && d == '+' || d == '-' {
                if range_has_sign {
                    invalid_range = true;
                }

                range_has_sign = true;
                lexeme.push(d);
                chars.next();
                *col += 1;
            } else {
                break;
            }
        }
    }

    if range_has_sign && !has_digits_after_dot {
        invalid_range = true;
    }

    if invalid_range {
        let err_msg = Some(format! {"'{lexeme}'"});
        print_syntax_err(
            "invalid format for range expression (format: <INT_MIN>..[=]<INT_MAX | '*'>)",
            err_msg,
            *line,
            *col,
            path,
        );
    }

    if range_has_dot && !has_digits_after_dot {
        lexeme.push('*');
    }

    if range_has_dot {
        return Ok(Token {
            kind: if range_has_eq {
                TokenKind::InclusiveRange
            } else {
                TokenKind::ExclusiveRange
            },
            lexeme,
            line: start_line,
            col: start_col,
        });
    }

    if begins_with_star {
        let prefix = if is_float { "float" } else { "int" };
        let err_msg = Some(format!("'{lexeme}'"));
        print_syntax_err(
            &format!("{prefix} cannot contain wildcard"),
            err_msg,
            *line,
            *col,
            path,
        );
    }

    if !has_digits_before_dot && has_sign {
        let err_msg = Some(format! {"'{lexeme}'"});
        print_syntax_err(
            "sign(s) with no associated number",
            err_msg,
            *line,
            *col,
            path,
        );
        return Err("");
    }

    // Invalid float: dot but no digits on one side
    if is_float && (!has_digits_before_dot || !has_digits_after_dot) {
        let err_msg = Some(format!("'{lexeme}'"));
        print_syntax_err(
            "float must have digits on both sides of decimal point",
            err_msg,
            *line,
            *col,
            path,
        );
        return Err("");
    }

    let mut has_exp = false;
    if let Some(&e) = chars.peek() {
        if e.eq_ignore_ascii_case(&'e') {
            is_float = true;
            lexeme.push(e);
            chars.next();
            *col += 1;

            if let Some(&sign) = chars.peek() {
                if sign == '+' || sign == '-' {
                    lexeme.push(sign);
                    chars.next();
                    *col += 1;
                }
            }

            let mut unrecognized_exp: Option<char> = None;
            while let Some(&n) = chars.peek() {
                if n.is_ascii_digit() || n == '_' {
                    has_exp = true;
                    chars.next();
                    lexeme.push(n);
                    *col += 1;
                } else if is_special_symbol(n) || n.is_whitespace() {
                    break;
                } else {
                    unrecognized_exp = Some(n);
                    break;
                }
            }

            if !has_exp {
                let err_msg = Some(format!("'{lexeme}'"));
                print_syntax_err("missing float exponent", err_msg, *line, *col, path);
            }

            if let Some(c) = unrecognized_exp {
                let err_msg = Some(format!("'{lexeme}{c}' ('{c}')"));
                print_syntax_err(
                    "float exponent contains illegal char(s)",
                    err_msg,
                    *line,
                    *col,
                    path,
                );
                return Err("");
            }
        }
    }

    Ok(Token {
        kind: if is_float {
            TokenKind::Float
        } else {
            TokenKind::Int
        },
        lexeme,
        line: start_line,
        col: start_col,
    })
}

fn parse_prefixed_int<F>(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    col: &mut usize,
    line: usize,
    start_col: usize,
    prefix: &str,
    is_valid: F,
    path: &str,
) -> Result<Token, &'static str>
where
    F: Fn(char) -> bool,
{
    let mut lexeme = format!("0{}", prefix);
    chars.next();
    *col += 1;

    let mut has_digit = false;
    while let Some(&ch) = chars.peek() {
        if is_valid(ch) || ch == '_' {
            if is_valid(ch) {
                has_digit = true;
            }
            lexeme.push(ch);
            chars.next();
            *col += 1;
        } else {
            break;
        }
    }

    if !has_digit {
        let err_msg = Some(format!("'{lexeme}"));
        print_syntax_err("expected digits after prefix", err_msg, line, *col, path);
        return Err("");
    }

    Ok(Token {
        kind: TokenKind::Int,
        lexeme,
        line,
        col: start_col,
    })
}
