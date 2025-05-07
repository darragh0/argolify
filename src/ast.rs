use crate::common::tokens::TokenKind;

#[derive(Debug)]
pub struct Statement {
    pub key: String,
    pub value: String,
    pub kind: TokenKind,
    pub ident_kind: TokenKind,
}

#[derive(Debug)]
pub struct Block {
    pub keyword: String,
    pub identifiers: Vec<String>,
    pub statements: Vec<Statement>,
    pub blocks: Vec<Block>,
}
