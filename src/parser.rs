use crate::{
    ast::{Block, Statement},
    common::errors::{
        Loc, ParseError, ParseError::*, SemanticError, SyntaxError, print_semantic_tip,
    },
    common::tokens::{Token, TokenKind, fmt_statement_kinds, is_statement_kind},
    tokenizer::tokenize,
};
use std::collections::HashSet;
use std::fs;
use std::iter::Peekable;
use std::path::Path;
use std::slice::Iter;

#[derive(Debug)]
pub struct Parser<'a> {
    pub root: Block,
    pub tokens: Peekable<Iter<'a, Token>>,
    pub path: String,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token], path: &Path) -> Self {
        let tokens = tokens.iter().peekable();
        let root = Block {
            keyword: "root".into(),
            identifiers: Vec::new(),
            statements: Vec::new(),
            blocks: Vec::new(),
        };

        Parser {
            root,
            tokens,
            path: path.to_string_lossy().to_string(),
        }
    }

    fn parse(&mut self) -> Result<(), ParseError> {
        let mut has_tok = false;
        while let Some(&tok) = self.tokens.peek() {
            has_tok = true;
            match tok.kind {
                TokenKind::Directive => self.parse_statement(tok, true, None)?,
                TokenKind::Keyword => self.parse_block(tok, None)?,
                _ => {
                    let loc = self.get_loc(tok);
                    let msg = format!(
                        "Expected {} or {}, but got {}: {}",
                        TokenKind::Directive,
                        TokenKind::Keyword,
                        tok.kind,
                        tok,
                    );
                    return Err(Syntax(SyntaxError::UnexpectedToken(loc, msg)));
                }
            }
        }

        if has_tok {
            Ok(())
        } else {
            let loc = Loc {
                path: self.path.to_string(),
                line: 0,
                col: 0,
            };
            let msg: String = format!(
                "Expected {} or {} but got no token",
                TokenKind::Directive,
                TokenKind::Keyword
            );
            Err(Syntax(SyntaxError::NoToken(loc, msg)))
        }
    }

    fn parse_block_header(&mut self, block: &mut Block) -> Result<(), ParseError> {
        self.expect_next(
            TokenKind::Identifier,
            format!(
                "Expected at least 1 {} after {}",
                TokenKind::Identifier,
                TokenKind::Keyword
            ),
        )?;

        let mut exp_alt = true;
        let mut short_names: HashSet<String> = HashSet::new();
        let mut unique_names: HashSet<String> = HashSet::new();
        let mut duplicate_names: HashSet<String> = HashSet::new();
        let mut error_loc: Option<Loc> = None;

        while let Some(&tok) = self.tokens.peek() {
            if exp_alt && tok.kind != TokenKind::Identifier {
                let msg = format!(
                    "Expected {} after {}, but got {}: {}",
                    TokenKind::Identifier,
                    TokenKind::Alt,
                    tok.kind,
                    tok
                );
                let loc = self.get_loc(tok);
                return Err(Syntax(SyntaxError::UnexpectedToken(loc, msg)));
            }

            if exp_alt {
                exp_alt = false;
                self.tokens.next();

                if tok.lexeme.len() == 1 && short_names.len() == 1 {
                    error_loc = Some(self.get_loc(tok));
                    short_names.insert(tok.lexeme.clone());
                } else if tok.lexeme.len() == 1 {
                    short_names.insert(tok.lexeme.clone());
                } else if unique_names.contains(&tok.lexeme) && duplicate_names.is_empty() {
                    error_loc = Some(self.get_loc(tok));
                    duplicate_names.insert(tok.lexeme.clone());
                } else if unique_names.contains(&tok.lexeme) {
                    duplicate_names.insert(tok.lexeme.clone());
                } else {
                    unique_names.insert(tok.lexeme.clone());
                }

                block.identifiers.push(tok.lexeme.clone());
            } else if tok.kind == TokenKind::OpenBrace {
                self.tokens.next();
                break;
            } else if tok.kind == TokenKind::Alt {
                exp_alt = true;
                self.tokens.next();
            } else {
                let msg = format!(
                    "Expected {} or {}, but got {}: {}",
                    TokenKind::Alt,
                    TokenKind::OpenBrace,
                    tok.kind,
                    tok
                );

                let loc = self.get_loc(tok);
                return Err(Syntax(SyntaxError::UnexpectedToken(loc, msg)));
            }
        }

        if short_names.len() > 1 {
            Err(Semantic(SemanticError::ExtraShortName(
                error_loc.unwrap(),
                short_names,
            )))
        } else if !duplicate_names.is_empty() {
            Err(Semantic(SemanticError::DuplicateName(
                error_loc.unwrap(),
                duplicate_names,
            )))
        } else {
            Ok(())
        }
    }

    fn parse_block(
        &mut self,
        kw_tok: &Token,
        parent: Option<&mut Block>,
    ) -> Result<(), ParseError> {
        let mut block = Block {
            keyword: kw_tok.lexeme.clone(),
            identifiers: Vec::new(),
            statements: Vec::new(),
            blocks: Vec::new(),
        };

        self.parse_block_header(&mut block)?;

        let mut empty_block = false;

        while let Some(&tok) = self.tokens.peek() {
            match tok.kind {
                TokenKind::CloseBrace => {
                    if empty_block {
                        let loc = self.get_loc(tok);
                        print_semantic_tip(&loc, "Redundant argument: empty block".into());
                    }

                    self.tokens.next();
                    break;
                }
                TokenKind::Identifier => {
                    self.parse_statement(tok, false, Some(&mut block))?;
                    empty_block = false;
                }
                TokenKind::Keyword => {
                    let loc = self.get_loc(tok);
                    if kw_tok.lexeme != "command" {
                        return Err(Semantic(SemanticError::CannotNest(loc, tok.lexeme.clone())));
                    }
                    self.parse_block(tok, Some(&mut block))?;
                    empty_block = false;
                }
                _ => {
                    let msg = if block.keyword == "command" {
                        format!(
                            "Expected {} or {}, but got {}: {}",
                            TokenKind::Identifier,
                            TokenKind::Keyword,
                            tok.kind,
                            tok,
                        )
                    } else {
                        format!(
                            "Expected {}, but got {}: {}",
                            TokenKind::Identifier,
                            tok.kind,
                            tok
                        )
                    };
                    let loc = self.get_loc(tok);
                    return Err(Syntax(SyntaxError::UnexpectedToken(loc, msg)));
                }
            }
        }

        if let Some(p) = parent {
            p.blocks.push(block);
        } else {
            self.root.blocks.push(block);
        }

        Ok(())
    }

    fn parse_statement(
        &mut self,
        ident_tok: &Token,
        is_dir: bool,
        parent: Option<&mut Block>,
    ) -> Result<(), ParseError> {
        let ident_kind = if is_dir {
            TokenKind::Directive
        } else {
            TokenKind::Identifier
        };

        let mut prev_tok = self.expect_next(
            TokenKind::Eq,
            format!("Expected {} after {}", TokenKind::Eq, ident_kind),
        )?;
        self.tokens.next();

        let mut str_vals: Vec<String> = Vec::new();
        let mut val_kind: Option<TokenKind> = None;
        let mut statement: Option<Statement> = None;

        while let Some(&tok) = self.tokens.peek() {
            let is_str = tok.kind == TokenKind::String;
            if is_dir && !is_str {
                let loc = self.get_loc(tok);
                let msg = format!(
                    "Expected {} after {} for a {} assignment, but got {}: {}",
                    TokenKind::String,
                    TokenKind::Eq,
                    ident_kind,
                    tok.kind,
                    tok
                );
                return Err(Syntax(SyntaxError::UnexpectedToken(loc, msg)));
            } else if is_dir && is_str {
                self.expect_semicolon()?;
                self.tokens.next();

                statement = Some(Statement {
                    key: ident_tok.lexeme.clone(),
                    value: tok.lexeme.clone(),
                    ident_kind,
                    kind: TokenKind::String,
                });
                break;
            }

            if tok.kind == TokenKind::Comma {
                prev_tok = tok.clone();
                self.tokens.next();
                continue;
            }

            if !is_statement_kind(&tok.kind) && prev_tok.kind == TokenKind::Comma {
                let loc = self.get_loc(tok);
                if let Some(vkind) = val_kind {
                    let msg = format!(
                        "Expected {} value after comma, but got {}: {}",
                        vkind, tok.kind, tok
                    );
                    return Err(Syntax(SyntaxError::UnexpectedToken(loc, msg)));
                } else {
                    let msg = format!("Expected value after comma, but got {}: {}", tok.kind, tok);
                    return Err(Syntax(SyntaxError::UnexpectedToken(loc, msg)));
                };
            }

            let is_sc = tok.kind == TokenKind::SemiColon;
            if is_sc && !str_vals.is_empty() {
                statement = Some(Statement {
                    key: ident_tok.lexeme.clone(),
                    value: str_vals.join(","),
                    kind: val_kind.unwrap(),
                    ident_kind,
                });
                self.tokens.next();
                break;
            }

            if let Some(vkind) = val_kind {
                if !is_sc && tok.kind != vkind {
                    if prev_tok.kind == TokenKind::Comma {
                        let loc = self.get_loc(tok);
                        let msg = format!(
                            "Expected {} value after comma, but got {}: {}",
                            vkind, tok.kind, tok
                        );
                        return Err(Semantic(SemanticError::DiffOptionTypes(loc, msg)));
                    } else {
                        let mut loc = self.get_loc(&prev_tok);
                        loc.col += prev_tok.len() - 1;
                        let msg = format!(
                            "Expected comma or semicolon after {} value, but got {}",
                            vkind, tok.kind
                        );

                        return Err(Syntax(SyntaxError::UnexpectedToken(loc, msg)));
                    };
                }
            } else {
                val_kind = Some(tok.kind);
            }

            if !is_statement_kind(&tok.kind) {
                let loc = self.get_loc(tok);
                let msg = format!(
                    "Expected one of the following after {} for a {} assignment (got {}): {}",
                    prev_tok.kind,
                    ident_kind,
                    &tok.kind,
                    fmt_statement_kinds()
                );

                return Err(Syntax(SyntaxError::UnexpectedToken(loc, msg)));
            }

            if prev_tok.kind == TokenKind::Comma && str_vals.is_empty() {
                let loc = self.get_loc(&prev_tok);
                let msg = format!("Redundant comma before value in {} assignment", ident_kind);
                print_semantic_tip(&loc, msg);
            }

            prev_tok = tok.clone();
            str_vals.push(tok.lexeme.clone());
            self.tokens.next();
        }

        if let Some(p) = parent {
            p.statements.push(statement.unwrap());
        } else {
            self.root.statements.push(statement.unwrap());
        }

        Ok(())
    }

    fn expect_semicolon(&mut self) -> Result<(), ParseError> {
        self.expect_next(
            TokenKind::SemiColon,
            format!(
                "Missing {} to end assignment statement",
                TokenKind::SemiColon,
            ),
        )?;
        Ok(())
    }

    fn expect_next(&mut self, exp: TokenKind, msg: String) -> Result<Token, ParseError> {
        let cur_tok = self.tokens.next();
        let expect_semicolon = exp == TokenKind::SemiColon;

        if let Some(&next_tok) = self.tokens.peek() {
            if next_tok.kind != exp {
                let mut loc: Loc;
                if expect_semicolon {
                    let cur_tok = cur_tok.unwrap();
                    loc = self.get_loc(cur_tok);
                    loc.col += cur_tok.len() - 1;
                } else {
                    loc = self.get_loc(next_tok);
                }

                let msg = format!("{msg}; next token is {}", next_tok.kind);
                return Err(Syntax(SyntaxError::UnexpectedToken(loc, msg)));
            }

            return Ok(next_tok.clone());
        }

        let loc = if let Some(t) = cur_tok {
            self.get_loc(t)
        } else {
            Loc {
                path: self.path.to_string(),
                line: 0,
                col: 0,
            }
        };

        let msg = format!("{msg}; got no token");
        Err(Syntax(SyntaxError::NoToken(loc, msg)))
    }

    fn get_loc(&self, tok: &Token) -> Loc {
        Loc {
            path: self.path.to_string(),
            line: tok.line,
            col: tok.col,
        }
    }
}

pub fn parse(path: &Path) -> Result<(), ParseError> {
    let content = fs::read_to_string(path)
        .map_err(|_| "Could not read file".to_string())
        .unwrap();

    let tokens = tokenize(&content, path)?;
    let mut parser = Parser::new(&tokens, path);

    parser.parse()?;

    println!("{:#?}", parser.root);

    Ok(())
}
