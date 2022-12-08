use std::{fmt::Display, rc::Rc};

use crate::{
    parser::tokenizer::Token,
    parser::{Parser, ParserResult},
    vm::VM,
};

use super::stmt::Stmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ExprType {
    // Literals
    Real(f64),
    Bool(bool),
    Null,
    String(Rc<String>),
    // Binary operations
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
    // Unary operations
    Negate(Box<Expr>),
    Not(Box<Expr>),
    // Other
    Grouping(Box<Expr>),
    Object(Vec<(String, Box<Expr>)>),
    Var(Rc<String>),
    Assign(Rc<String>, Box<Expr>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub token: Token,
    pub kind: ExprType,
}

impl Expr {
    pub fn new(token: Token, kind: ExprType) -> Self {
        Self { token, kind }
    }

    pub fn parse(tokens: Vec<Token>, source: Vec<char>) -> ParserResult<Expr> {
        let mut parser = Parser::new(tokens, source);
        parser.expression()
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ExprType::Real(a) => write!(f, "{}", a),
            ExprType::Bool(a) => write!(f, "{}", a),
            ExprType::Null => write!(f, "null"),
            ExprType::Add(l, r) => write!(f, "(+ {} {})", l, r),
            ExprType::Subtract(l, r) => write!(f, "(- {} {})", l, r),
            ExprType::Multiply(l, r) => write!(f, "(* {} {})", l, r),
            ExprType::Divide(l, r) => write!(f, "(/ {} {})", l, r),
            ExprType::Negate(inner) => write!(f, "(- {})", inner),
            ExprType::Not(inner) => write!(f, "(! {})", inner),
            ExprType::Grouping(inner) => write!(f, "{}", inner),
            ExprType::Object(table) => write!(f, "{:?}", table),
            ExprType::Var(v) => write!(f, "(get {})", v),
            ExprType::Assign(name, value) => write!(f, "(set {} to {:?})", name, value),
            ExprType::String(a) => write!(f, "({:?})", a),
        }
    }
}

pub trait AstVisitor<T, A> {
    fn visit_stmt(&mut self, stmt: &Stmt, vm: &VM) -> A;

    fn visit_node(&mut self, node: &Expr, vm: &VM) -> T;
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::expr::Expr,
        parser::tokenizer::Tokenizer,
        parser::{ParserErrorType, ParserResult},
    };
    fn parse_expr<S: AsRef<str>>(source: S) -> ParserResult<Expr> {
        let tokens = Tokenizer::new(source.as_ref())
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let ast = Expr::parse(tokens, source.as_ref().chars().collect());

        ast
    }
    fn parse_expr_lisp<S: AsRef<str>>(source: S) -> String {
        let tokens = Tokenizer::new(source.as_ref())
            .map(|v| v.unwrap())
            .collect::<Vec<_>>();

        let ast = Expr::parse(tokens, source.as_ref().chars().collect());

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
        assert_eq!(err.kind, ParserErrorType::ExpectedExpression)
    }
}
