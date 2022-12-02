use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::{
    ast::{AstNode, AstType},
    error::EscuroError,
    tokenizer::{Token, TokenType},
};
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
        write!(f, "{}", self.msg())
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
    FailedToMatchAnyRules,
}
impl EscuroError for ParserError {
    fn msg(&self) -> &str {
        match self.kind {
            ParserErrorType::RealParseFailed => "parsing real failed",
            ParserErrorType::UnclosedParentheses => "unclosed parentheses",
            ParserErrorType::FailedToMatchAnyRules => "matches no rules",
        }
    }
    fn code(&self) -> u32 {
        match self.kind {
            ParserErrorType::RealParseFailed => 2002,
            ParserErrorType::UnclosedParentheses => 2003,
            ParserErrorType::FailedToMatchAnyRules => 2001,
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
    panic_mode: bool,
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
    pub fn expression(&mut self) -> ParserResult<AstNode> {
        self.equality()
    }

    pub fn equality(&mut self) -> ParserResult<AstNode> {
        let mut e = self.comparison()?;

        while self.mtch(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.prev();
            let right = self.comparison()?;
            e = self.binop(op, e, right);
        }
        Ok(e)
    }

    pub fn comparison(&mut self) -> ParserResult<AstNode> {
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
    pub fn term(&mut self) -> ParserResult<AstNode> {
        let mut e = self.factor()?;
        while self.mtch(&[TokenType::Minus, TokenType::Plus]) {
            let op = self.prev();
            let right = self.factor()?;
            e = self.binop(op, e, right)
        }
        Ok(e)
    }
    pub fn factor(&mut self) -> ParserResult<AstNode> {
        let mut e = self.unary()?;
        while self.mtch(&[TokenType::Slash, TokenType::Star]) {
            let op = self.prev();
            let right = self.unary()?;
            e = self.binop(op, e, right)
        }
        Ok(e)
    }
    pub fn unary(&mut self) -> ParserResult<AstNode> {
        if self.mtch(&[TokenType::Bang, TokenType::Star]) {
            let op = self.prev();
            let inner = self.unary()?;
            return Ok(self.unop(op, inner));
        }
        self.primary()
    }
    pub fn primary(&mut self) -> ParserResult<AstNode> {
        if self.mtch(&[TokenType::False]) {
            return Ok(AstNode::new(self.prev(), AstType::Bool(false)));
        }
        if self.mtch(&[TokenType::True]) {
            return Ok(AstNode::new(self.prev(), AstType::Bool(true)));
        }
        if self.mtch(&[TokenType::Null]) {
            return Ok(AstNode::new(self.prev(), AstType::Null));
        }
        if self.mtch(&[TokenType::Number]) {
            let a = self.source[self.prev().start..=self.prev().start + self.prev().length - 1]
                .iter()
                .collect::<String>();

            return Ok(AstNode::new(
                self.prev(),
                AstType::Real(a.parse::<f64>().map_err(|_| {
                    ParserError::new(ParserErrorType::RealParseFailed, self.prev())
                })?),
            ));
        }
        // TODO: string self.source[self.start + 1..=self.current - 2].iter().collect()

        if self.mtch(&[TokenType::LParen]) {
            let expr = self.expression()?;
            if self.peek().kind == TokenType::RParen {
                self.advance();
                return Ok(AstNode::new(self.prev(), AstType::Grouping(Box::new(expr))));
            } else {
                return Err(ParserError::new(
                    ParserErrorType::UnclosedParentheses,
                    self.peek(),
                ));
            }
        }

        Err(ParserError::new(
            ParserErrorType::FailedToMatchAnyRules,
            self.peek(),
        ))
    }
    fn binop(&self, op: Token, left: AstNode, right: AstNode) -> AstNode {
        match op.kind {
            TokenType::Plus => AstNode::new(op, AstType::Add(Box::new(left), Box::new(right))),
            TokenType::Minus => {
                AstNode::new(op, AstType::Subtract(Box::new(left), Box::new(right)))
            }
            TokenType::Star => AstNode::new(op, AstType::Multiply(Box::new(left), Box::new(right))),
            TokenType::Slash => AstNode::new(op, AstType::Divide(Box::new(left), Box::new(right))),
            _ => unimplemented!(),
        }
    }
    fn unop(&self, op: Token, inner: AstNode) -> AstNode {
        match op.kind {
            TokenType::Minus => AstNode::new(op, AstType::Negate(Box::new(inner))),
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
