use std::fmt::Display;

use crate::{
    parser::{Parser, ParserResult},
    tokenizer::Token,
};

#[derive(Clone, Debug, PartialEq)]
pub enum AstType {
    // Literals
    Real(f64),
    Bool(bool),
    Null,
    // Binary operations
    Add(Box<AstNode>, Box<AstNode>),
    Subtract(Box<AstNode>, Box<AstNode>),
    Multiply(Box<AstNode>, Box<AstNode>),
    Divide(Box<AstNode>, Box<AstNode>),
    // Unary operations
    Negate(Box<AstNode>),
    // Other
    Grouping(Box<AstNode>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct AstNode {
    pub token: Token,
    pub kind: AstType,
}

impl AstNode {
    pub fn new(token: Token, kind: AstType) -> Self {
        Self { token, kind }
    }

    pub fn parse(tokens: Vec<Token>, source: Vec<char>) -> ParserResult<AstNode> {
        let mut parser = Parser::new(tokens, source);
        parser.expression()
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstType::Real(a) => write!(f, "{}", a),
            AstType::Bool(a) => write!(f, "{}", a),
            AstType::Null => write!(f, "null"),
            AstType::Add(l, r) => write!(f, "(+ {} {})", l, r),
            AstType::Subtract(l, r) => write!(f, "(- {} {})", l, r),
            AstType::Multiply(l, r) => write!(f, "(* {} {})", l, r),
            AstType::Divide(l, r) => write!(f, "(/ {} {})", l, r),
            AstType::Negate(inner) => write!(f, "(- {})", inner),
            AstType::Grouping(inner) => write!(f, "{}", inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::AstNode,
        parser::{ParserError, ParserErrorType, ParserResult},
        tokenizer::Tokenizer,
    };
    fn parse_expr<S: AsRef<str>>(source: S) -> ParserResult<AstNode> {
        let tokens = Tokenizer::new(source.as_ref())
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let ast = AstNode::parse(tokens, source.as_ref().chars().collect());

        ast
    }
    fn parse_expr_lisp<S: AsRef<str>>(source: S) -> String {
        let tokens = Tokenizer::new(source.as_ref())
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let ast = AstNode::parse(tokens, source.as_ref().chars().collect());

        format!("{}", ast.unwrap())
    }
    #[test]
    fn binops() {
        let source = "(1 + 1) - (2 / (3 * 2))";
        let ast = parse_expr_lisp(source);
        assert_eq!(ast, "(- (+ 1 1) (/ 2 (* 3 2)))");
    }

    #[test]
    fn parse() {
        let source = "(";
        let ast = parse_expr(source);
        let err = ast.unwrap_err();
        // println!("{:?}", ast);
        assert_eq!(err.kind, ParserErrorType::FailedToMatchAnyRules)
    }
}
