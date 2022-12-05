use super::{expr::Expr, tokenizer::Token, Parser, ParserResult};

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub kind: StmtType,
}

impl Stmt {
    pub fn new(kind: StmtType) -> Self {
        Self { kind }
    }

    pub fn parse(tokens: Vec<Token>, source: Vec<char>) -> ParserResult<Vec<Stmt>> {
        let mut parser = Parser::new(tokens, source);
        let mut stmts = vec![];
        while let Ok(stmt) = parser.statement() {
            stmts.push(stmt);
        }
        Ok(stmts)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum StmtType {
    Print(Expr),
    Expr(Expr),
}
