pub mod expr;
pub mod stmt;
pub mod tokenizer;

use std::{
    backtrace::Backtrace,
    error::Error,
    fmt::{Debug, Display},
    rc::Rc,
};

use once_cell::unsync::OnceCell;

use crate::{
    parser::expr::{Expr, ExprType},
    parser::tokenizer::{Token, TokenType},
    util::error::AnkokuError,
};

use self::stmt::{Stmt, StmtType};
pub type ParserResult<T> = Result<T, ParserError>;
pub struct ParserError {
    pub kind: ParserErrorType,
    pub token: Token,
    pub internal_bt: Backtrace,
    pub line: String,
    pub line_num: u32,
    pub col: usize,
}
impl ParserError {
    pub fn new(kind: ParserErrorType, token: Token, line: String, line_col: (u32, usize)) -> Self {
        ParserError {
            kind,
            token,
            internal_bt: Backtrace::capture(),
            line,
            line_num: line_col.0,
            col: line_col.1,
        }
    }
}
impl Error for ParserError {}
impl Debug for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} on {:?}
        internal backtrace:
        {}",
            self.msg(),
            self.token,
            self.internal_bt
        )
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
    ExpectedSemicolon { after_variable: bool },
    ObjectNeedsIdentifierKeys,
    UnclosedObject,
    ExpectVariableName,
    ExpectEqualAfterIdentifierInObject,
    InvalidAssignmentTarget,
}
impl AnkokuError for ParserError {
    fn msg(&self) -> &str {
        match self.kind {
            ParserErrorType::RealParseFailed => "parsing real failed",
            ParserErrorType::UnclosedParentheses => "unclosed parentheses",
            ParserErrorType::ExpectedExpression => "expected expression",
            ParserErrorType::ExpectedSemicolon { after_variable } => {
                if after_variable {
                    "expected semicolon after variable declaration: ;"
                } else {
                    "expected semicolon: ;"
                }
            }
            ParserErrorType::ObjectNeedsIdentifierKeys => "object keys must be identifiers",
            ParserErrorType::UnclosedObject => "unclosed object, expected }",
            ParserErrorType::ExpectVariableName => "expected variable name after \"var\"",
            ParserErrorType::ExpectEqualAfterIdentifierInObject => {
                "expect equal after identifier in object literal, like: { meaning_of_life = 42 }"
            }
            ParserErrorType::InvalidAssignmentTarget => "invalid assignment target",
        }
    }
    fn code(&self) -> u32 {
        match self.kind {
            ParserErrorType::ExpectedExpression => 2001,
            ParserErrorType::RealParseFailed => 2002,
            ParserErrorType::UnclosedParentheses => 2003,
            ParserErrorType::ExpectedSemicolon { .. } => 2004,
            ParserErrorType::ObjectNeedsIdentifierKeys => 2005,
            ParserErrorType::UnclosedObject => 2006,
            ParserErrorType::ExpectVariableName => 2007,
            ParserErrorType::ExpectEqualAfterIdentifierInObject => 2008,
            ParserErrorType::InvalidAssignmentTarget => 2009,
        }
    }

    fn line_col(&self) -> Option<(u32, usize, &str)> {
        Some((self.line_num, self.col, &self.line))
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
    source_string: OnceCell<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, source: Vec<char>) -> Self {
        Self {
            tokens,
            source,
            current: 0,
            panic_mode: false,
            source_string: OnceCell::new(),
        }
    }

    fn idx_to_pos(&self, idx: usize) -> (u32, usize) {
        let mut col = 0;
        let mut lines = 0;
        for i in 0..idx {
            if self.source[i] == '\n' {
                lines += 1;
                col = 0;
                continue;
            }
            col += 1;
        }
        (lines + 1, col + 1)
    }

    fn get_line(&self, line_num: u32) -> String {
        assert!(line_num >= 1);
        let mut lines = self
            .source_string
            .get_or_init(|| String::from_iter(&self.source))
            .lines();

        lines
            .nth((line_num - 1) as usize)
            .expect("failed to get line")
            .to_string()
    }

    fn new_err(&self, kind: ParserErrorType, token: Token) -> ParserError {
        println!("{:?}", token);
        if token.kind == TokenType::EOF {
            ParserError::new(kind, token, "EOF".into(), (1, 1))
        } else {
            ParserError::new(
                kind,
                token,
                self.get_line(self.idx_to_pos(token.start).0),
                self.idx_to_pos(token.start),
            )
        }
    }
    pub fn declaration(&mut self) -> ParserResult<Stmt> {
        if self.mtch(&[TokenType::Var]) {
            self.var_decl()
        } else {
            self.statement()
        }
    }

    fn var_decl(&mut self) -> ParserResult<Stmt> {
        let global = self.parse_variable(ParserErrorType::ExpectVariableName)?;
        let expr = if self.mtch(&[TokenType::Equal]) {
            self.expression()
        } else {
            Ok(Expr::new(self.peek(), ExprType::Null))
        }?;
        self.expect_semi(Stmt::new(StmtType::Var(
            self.source[global.start..=global.start + global.length - 1]
                .iter()
                .collect::<String>(),
            expr,
        )))
    }

    fn parse_variable(&mut self, error: ParserErrorType) -> ParserResult<Token> {
        if self.peek().kind == TokenType::Identifier {
            Ok(self.advance())
        } else {
            Err(self.new_err(error, self.peek()))
        }
    }

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
            Err(self.new_err(
                ParserErrorType::ExpectedSemicolon {
                    after_variable: false,
                },
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
        match self.assignment() {
            Ok(a) => Ok(a),
            Err(err) => {
                self.panic_mode = true;
                Err(err)
            }
        }
    }

    fn assignment(&mut self) -> ParserResult<Expr> {
        let expr = self.equality()?;

        if self.mtch(&[TokenType::Equal]) {
            let equals = self.prev();
            let value = self.assignment()?;

            if let ExprType::Identifier(name) = expr.kind {
                return Ok(Expr::new(equals, ExprType::Assign(name, Box::new(value))));
            }

            return Err(self.new_err(ParserErrorType::InvalidAssignmentTarget, self.peek()));
        }

        Ok(expr)
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
        if self.mtch(&[TokenType::Identifier]) {
            let name = self.source[self.prev().start..=self.prev().start + self.prev().length - 1]
                .iter()
                .collect::<String>(); // TODO: implement string interner for this, not sure how it will work since UTF-32 &[char] != UTF-8 String
            return Ok(Expr::new(self.prev(), ExprType::Identifier(Rc::new(name))));
        }
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

            if self.mtch(&[TokenType::Dot]) {
                return Err(self.new_err(ParserErrorType::RealParseFailed, self.prev()));
            }

            return Ok(Expr::new(
                self.prev(),
                ExprType::Real(
                    a.parse::<f64>()
                        .map_err(|_| self.new_err(ParserErrorType::RealParseFailed, self.prev()))?,
                ),
            ));
        }

        if self.mtch(&[TokenType::String]) {
            let a = self.source[self.prev().start..=self.prev().start + self.prev().length - 1]
                .iter()
                .collect::<String>();

            if self.mtch(&[TokenType::Dot]) {
                return Err(self.new_err(ParserErrorType::RealParseFailed, self.prev()));
            }

            return Ok(Expr::new(self.prev(), ExprType::String(Rc::new(a)))); // maybe intern these i don't know
        }

        if self.mtch(&[TokenType::LParen]) {
            let expr = self.expression()?;
            println!("{:?}", expr);
            if self.peek().kind == TokenType::RParen {
                self.advance();
                return Ok(Expr::new(self.prev(), ExprType::Grouping(Box::new(expr))));
            } else {
                return Err(self.new_err(ParserErrorType::UnclosedParentheses, self.peek()));
            }
        }

        if self.mtch(&[TokenType::LBrace]) {
            return self.object();
        }

        Err(self.new_err(ParserErrorType::ExpectedExpression, self.peek()))
    }
    fn consume(&mut self, expect: TokenType, error: ParserErrorType) -> ParserResult<Token> {
        if self.peek().kind == expect {
            Ok(self.advance())
        } else {
            Err(self.new_err(error, self.peek()))
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        while self.peek().kind != TokenType::EOF {
            if self.current > 0 && self.prev().kind == TokenType::Semicolon {
                return;
            }

            match self.peek().kind {
                TokenType::Class
                | TokenType::Fn
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }

            self.advance();
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
            self.consume(
                TokenType::Equal,
                ParserErrorType::ExpectEqualAfterIdentifierInObject,
            )?;
            let value = self.expression()?;

            pairs.push((key, Box::new(value)));

            if self.peek().kind == TokenType::RBrace {
                self.advance();
                return Ok(Expr::new(start, ExprType::Object(pairs)));
            } else if self.peek().kind == TokenType::Comma {
                self.advance();
                continue;
            } else {
                return Err(self.new_err(ParserErrorType::UnclosedObject, self.peek()));
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
