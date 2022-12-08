use super::{expr::Expr, tokenizer::Token, Parser, ParserError};

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub kind: StmtType,
}

impl Stmt {
    pub fn new(kind: StmtType) -> Self {
        Self { kind }
    }

    pub fn parse(tokens: Vec<Token>, source: Vec<char>) -> (Vec<Stmt>, Vec<ParserError>) {
        let mut parser = Parser::new(tokens, source);
        let mut stmts = vec![];
        let mut errors = vec![];
        while !parser.at_end() {
            let stmt = parser.declaration();
            if let Ok(stmt) = stmt {
                stmts.push(stmt);
            } else if let Err(e) = stmt {
                errors.push(e);
                parser.synchronize();
            };
        }
        (stmts, errors)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum StmtType {
    Print(Expr),
    Expr(Expr),
    Var(String, Expr),
}
