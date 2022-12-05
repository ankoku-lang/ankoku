pub mod expr;
pub mod stmt;
pub mod tokenizer;

use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{
    parser::expr::{Expr, ExprType},
    parser::tokenizer::{Token, TokenType},
    util::error::AnkokuError,
    vm::{obj::AnkokuString, table::HashTable},
};

use self::stmt::{Stmt, StmtType};
pub type ParserResult<T> = Result<T, ParserError>;
#[derive(Clone)]
pub struct ParserError {
    pub kind: ParserErrorType,
    pub token: Token,
}
impl ParserError {
    pub fn new(kind: ParserErrorType, token: Token) -> Self {
        ParserError { kind, token }
    }
}
impl Error for ParserError {}
impl Debug for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} on {:?}", self.msg(), self.token)
    }
}
impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParserErrorType {
    RealParseFailed,
    UnclosedParentheses,
    ExpectedExpression,
    ExpectedSemicolon,
    ObjectNeedsIdentifierKeys,
    UnclosedObject,
}
impl AnkokuError for ParserError {
    fn msg(&self) -> &str {
        match self.kind {
            ParserErrorType::RealParseFailed => "parsing real failed",
            ParserErrorType::UnclosedParentheses => "unclosed parentheses",
            ParserErrorType::ExpectedExpression => "expected expression",
            ParserErrorType::ExpectedSemicolon => "expected semicolon: ;",
            ParserErrorType::ObjectNeedsIdentifierKeys => "object keys must be identifiers",
            ParserErrorType::UnclosedObject => "unclosed object, expected }",
        }
    }
    fn code(&self) -> u32 {
        match self.kind {
            ParserErrorType::ExpectedExpression => 2001,
            ParserErrorType::RealParseFailed => 2002,
            ParserErrorType::UnclosedParentheses => 2003,
            ParserErrorType::ExpectedSemicolon => 2004,
            ParserErrorType::ObjectNeedsIdentifierKeys => 2005,
            ParserErrorType::UnclosedObject => 2006,
        }
    }

    fn line_col(&self) -> Option<(u32, usize, &str)> {
        None
    }

    fn filename(&self) -> Option<&str> {
        None
    }

    fn length(&self) -> Option<usize> {
        None
    }
}
pub struct Parser {
    source: Vec<char>,
    tokens: Vec<Token>,
    current: usize,
    panic_mode: bool, // TODO
}

impl Parser {
    pub fn new(tokens: Vec<Token>, source: Vec<char>) -> Self {
        Self {
            tokens,
            source,
            current: 0,
            panic_mode: false,
        }
    }

    // TODO: errors for statements

    pub fn statement(&mut self) -> ParserResult<Stmt> {
        if self.mtch(&[TokenType::Print]) {
            return self.print_statement();
        }
        self.expression_statement()
    }

    fn expect_semi<T>(&mut self, a: T) -> ParserResult<T> {
        if self.peek().kind == TokenType::Semicolon {
            self.advance();
            Ok(a)
        } else {
            Err(ParserError::new(
                ParserErrorType::ExpectedSemicolon,
                self.peek(),
            ))
        }
    }

    fn expression_statement(&mut self) -> ParserResult<Stmt> {
        let stmt = Stmt::new(StmtType::Expr(self.expression()?));

        self.expect_semi(stmt)
    }

    fn print_statement(&mut self) -> ParserResult<Stmt> {
        let stmt = Stmt::new(StmtType::Print(self.expression()?));

        self.expect_semi(stmt)
    }

    pub fn expression(&mut self) -> ParserResult<Expr> {
        match self.equality() {
            Ok(a) => Ok(a),
            Err(err) => {
                self.panic_mode = true;
                Err(err)
            }
        }
    }

    pub fn equality(&mut self) -> ParserResult<Expr> {
        let mut e = self.comparison()?;

        while self.mtch(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.prev();
            let right = self.comparison()?;
            e = self.binop(op, e, right);
        }
        Ok(e)
    }

    pub fn comparison(&mut self) -> ParserResult<Expr> {
        let mut e = self.term()?;
        while self.mtch(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.prev();
            let right = self.term()?;
            e = self.binop(op, e, right)
        }
        Ok(e)
    }
    pub fn term(&mut self) -> ParserResult<Expr> {
        let mut e = self.factor()?;
        while self.mtch(&[TokenType::Minus, TokenType::Plus]) {
            let op = self.prev();
            let right = self.factor()?;
            e = self.binop(op, e, right)
        }
        Ok(e)
    }
    pub fn factor(&mut self) -> ParserResult<Expr> {
        let mut e = self.unary()?;
        while self.mtch(&[TokenType::Slash, TokenType::Star]) {
            let op = self.prev();
            let right = self.unary()?;
            e = self.binop(op, e, right)
        }
        Ok(e)
    }
    pub fn unary(&mut self) -> ParserResult<Expr> {
        if self.mtch(&[TokenType::Bang, TokenType::Minus]) {
            let op = self.prev();
            let inner = self.unary()?;
            return Ok(self.unop(op, inner));
        }
        self.primary()
    }
    pub fn primary(&mut self) -> ParserResult<Expr> {
        if self.mtch(&[TokenType::False]) {
            return Ok(Expr::new(self.prev(), ExprType::Bool(false)));
        }
        if self.mtch(&[TokenType::True]) {
            return Ok(Expr::new(self.prev(), ExprType::Bool(true)));
        }
        if self.mtch(&[TokenType::Null]) {
            return Ok(Expr::new(self.prev(), ExprType::Null));
        }
        if self.mtch(&[TokenType::Number]) {
            let a = self.source[self.prev().start..=self.prev().start + self.prev().length - 1]
                .iter()
                .collect::<String>();

            return Ok(Expr::new(
                self.prev(),
                ExprType::Real(a.parse::<f64>().map_err(|_| {
                    ParserError::new(ParserErrorType::RealParseFailed, self.prev())
                })?),
            ));
        }
        // TODO: string self.source[self.start + 1..=self.current - 2].iter().collect()

        if self.mtch(&[TokenType::LParen]) {
            let expr = self.expression()?;
            if self.peek().kind == TokenType::RParen {
                self.advance();
                return Ok(Expr::new(self.prev(), ExprType::Grouping(Box::new(expr))));
            } else {
                return Err(ParserError::new(
                    ParserErrorType::UnclosedParentheses,
                    self.peek(),
                ));
            }
        }

        if self.mtch(&[TokenType::LBrace]) {
            return self.object();
        }

        Err(ParserError::new(
            ParserErrorType::ExpectedExpression,
            self.peek(),
        ))
    }
    fn consume(&mut self, expect: TokenType, error: ParserErrorType) -> ParserResult<Token> {
        if self.peek().kind == expect {
            Ok(self.advance())
        } else {
            Err(ParserError::new(error, self.peek()))
        }
    }
    fn object(&mut self) -> ParserResult<Expr> {
        let mut pairs = Vec::new();
        let start = self.prev();
        loop {
            self.consume(
                TokenType::Identifier,
                ParserErrorType::ObjectNeedsIdentifierKeys,
            )?;
            let key = self.source[self.prev().start..=self.prev().start + self.prev().length - 1]
                .iter()
                .collect::<String>();
            println!(
                "{}",
                self.source[self.prev().start..=self.prev().start + self.prev().length - 1]
                    .iter()
                    .collect::<String>()
            );
            self.consume(TokenType::Equal, ParserErrorType::ExpectedExpression)?; // FIXME: create new error type for this
            let value = self.expression()?;

            pairs.push((key, Box::new(value)));

            if self.peek().kind == TokenType::RBrace {
                self.advance();
                return Ok(Expr::new(start, ExprType::Object(pairs)));
            } else if self.peek().kind == TokenType::Comma {
                self.advance();
                continue;
            } else {
                return Err(ParserError::new(
                    ParserErrorType::UnclosedObject,
                    self.peek(),
                ));
            }
        }
    }
    fn binop(&self, op: Token, left: Expr, right: Expr) -> Expr {
        match op.kind {
            TokenType::Plus => Expr::new(op, ExprType::Add(Box::new(left), Box::new(right))),
            TokenType::Minus => Expr::new(op, ExprType::Subtract(Box::new(left), Box::new(right))),
            TokenType::Star => Expr::new(op, ExprType::Multiply(Box::new(left), Box::new(right))),
            TokenType::Slash => Expr::new(op, ExprType::Divide(Box::new(left), Box::new(right))),
            _ => unimplemented!(),
        }
    }
    fn unop(&self, op: Token, inner: Expr) -> Expr {
        match op.kind {
            TokenType::Minus => Expr::new(op, ExprType::Negate(Box::new(inner))),
            TokenType::Bang => Expr::new(op, ExprType::Not(Box::new(inner))),
            _ => unimplemented!(),
        }
    }
    pub fn mtch(&mut self, types: &[TokenType]) -> bool {
        for typ in types {
            if self.check(*typ) {
                self.advance();
                return true;
            }
        }
        false
    }
    pub fn check(&mut self, kind: TokenType) -> bool {
        if self.at_end() {
            false
        } else {
            self.peek().kind == kind
        }
    }
    fn advance(&mut self) -> Token {
        self.current += 1;
        self.tokens[self.current - 1]
    }
    fn peek(&self) -> Token {
        self.tokens[self.current]
    }
    fn prev(&self) -> Token {
        self.tokens[self.current - 1]
    }
    fn at_end(&self) -> bool {
        self.peek().kind == TokenType::EOF
    }
}
