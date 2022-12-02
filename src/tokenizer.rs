use std::{
    error::Error,
    fmt::{Debug, Display},
};

use once_cell::unsync::OnceCell;

use crate::error::EscuroError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    BitwiseAnd,
    BitwiseOr,
    And,
    Or,
    String,
    Number,
    Identifier,
    Class,
    Else,
    False,
    For,
    Fn,
    If,
    Null,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    EOF,
}
pub type TokenizerResult<T> = Result<T, TokenizerError>;
#[derive(Clone)]
pub struct TokenizerError {
    pub kind: TokenizerErrorType,
    pub line: String,
    pub line_num: u32,
}
impl TokenizerError {
    pub fn new(kind: TokenizerErrorType, line: String, line_num: u32) -> Self {
        assert!(line_num >= 1, "line numbers start at 1");
        TokenizerError {
            kind,
            line,
            line_num,
        }
    }
}
impl Error for TokenizerError {}
impl Debug for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg())
    }
}
impl Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg())
    }
}
#[derive(Clone)]
pub enum TokenizerErrorType {
    UnexpectedCharacter,
    UnterminatedString,
}
impl EscuroError for TokenizerError {
    fn msg(&self) -> &str {
        match self.kind {
            TokenizerErrorType::UnexpectedCharacter => "unexpected character",
            TokenizerErrorType::UnterminatedString => "unterminated string (missing closing \")",
        }
    }

    fn line(&self) -> Option<(u32, &str)> {
        Some((self.line_num, &self.line))
    }

    fn filename(&self) -> Option<&str> {
        todo!()
    }

    fn code(&self) -> u32 {
        match self.kind {
            TokenizerErrorType::UnexpectedCharacter => 1000,
            TokenizerErrorType::UnterminatedString => 1001,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenType,
    pub start: usize, // start character
    pub length: usize,
    pub line: u32,
}

impl Token {
    pub fn new(kind: TokenType, start: usize, length: usize, line: u32) -> Self {
        Self {
            kind,
            start,
            length,
            line,
        }
    }
}

pub struct Tokenizer {
    pub(crate) source: Vec<char>,
    start: usize,
    current: usize,
    line: u32,
    done: bool,
    source_string: OnceCell<String>,
}
impl Tokenizer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            start: 0,
            done: false,
            source_string: OnceCell::new(),
        }
    }
    pub fn at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    pub fn next_token(&mut self) -> TokenizerResult<Token> {
        self.skip_whitespace();
        self.start = self.current;
        if self.at_end() {
            return Ok(self.new_token(TokenType::EOF));
        }
        let c = self.advance();
        if c.is_alphabetic() {
            let kind = self.identifier();
            return Ok(self.new_token(kind));
        }
        let eqm = self.mtch('=');
        match c {
            '(' => return Ok(self.new_token(TokenType::LParen)),
            ')' => return Ok(self.new_token(TokenType::RParen)),
            '{' => return Ok(self.new_token(TokenType::LBrace)),
            '}' => return Ok(self.new_token(TokenType::RBrace)),
            ';' => return Ok(self.new_token(TokenType::Semicolon)),
            ',' => return Ok(self.new_token(TokenType::Dot)),
            '-' => return Ok(self.new_token(TokenType::Minus)),
            '+' => return Ok(self.new_token(TokenType::Plus)),
            '/' => {
                return Ok(self.new_token(TokenType::Slash));
            }
            '*' => return Ok(self.new_token(TokenType::Star)),
            '!' => {
                return Ok(self.new_token(if eqm {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }))
            }
            '=' => {
                return Ok(self.new_token(if eqm {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }))
            }
            '<' => {
                return Ok(self.new_token(if eqm {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }))
            }
            '>' => {
                return Ok(self.new_token(if eqm {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }))
            }
            '&' => {
                return Ok(self.new_token(if eqm {
                    TokenType::And
                } else {
                    TokenType::BitwiseAnd
                }))
            }
            '|' => {
                return Ok(self.new_token(if eqm {
                    TokenType::Or
                } else {
                    TokenType::BitwiseOr
                }))
            }
            '"' => {
                return self.string();
            }

            '0'..='9' => {
                return Ok(self.number());
            }
            _ => {}
        }

        Err(TokenizerError::new(
            TokenizerErrorType::UnexpectedCharacter,
            self.get_line(self.line),
            self.line,
        ))
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
    fn number(&mut self) -> Token {
        while self.peek().map_or(false, |v| v.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some('.') && self.peek_next().map_or(false, |v| v.is_ascii_digit()) {
            self.advance();

            while self.peek().map_or(false, |v| v.is_ascii_digit()) {
                self.advance();
            }
        }
        self.new_token(TokenType::Number)
    }
    fn string(&mut self) -> TokenizerResult<Token> {
        while self.peek() != Some('"') && !self.at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }
        if self.at_end() {
            return Err(TokenizerError::new(
                TokenizerErrorType::UnterminatedString,
                self.get_line(self.line),
                self.line,
            ));
        }
        self.advance();
        Ok(self.new_token(TokenType::String))
    }
    fn identifier(&mut self) -> TokenType {
        while self
            .peek()
            .map_or(false, |v| v.is_alphanumeric() || v == '_')
        {
            self.advance();
        }

        let ident = self.source[self.start..=self.current - 1]
            .iter()
            .collect::<String>();
        match ident.as_str() {
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "if" => TokenType::If,
            "null" => TokenType::Null,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fn" => TokenType::Fn,
            "this" => TokenType::This,
            "true" => TokenType::True,
            _ => TokenType::Identifier,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.peek().map_or(false, |v| v.is_whitespace() || v == '/') {
            if self.peek().unwrap() == '\n' {
                self.line += 1;
            }
            if self.peek().unwrap() == '/' && self.peek_next() == Some('/') {
                while !self.peek().map_or(true, |v| v == '\n') {
                    self.advance();
                }
            } else if self.peek().unwrap() == '/' {
                return;
            }
            if !self.at_end() {
                self.advance();
            }
        }
    }
    fn mtch(&mut self, expected: char) -> bool {
        if self.at_end() {
            return false;
        }
        if self.peek() != Some(expected) {
            return false;
        }
        self.advance();
        true
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }
    fn peek(&self) -> Option<char> {
        if self.current >= self.source.len() {
            None
        } else {
            Some(self.source[self.current])
        }
    }
    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            None
        } else {
            Some(self.source[self.current + 1])
        }
    }

    fn new_token(&self, kind: TokenType) -> Token {
        Token {
            kind,
            length: self.current - self.start,
            line: self.line,
            start: self.start,
        }
    }
}
impl Iterator for Tokenizer {
    type Item = TokenizerResult<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let x = self.next_token();
            if let Ok(e) = &x {
                if e.kind == TokenType::EOF {
                    self.done = true
                }
            } else {
                self.done = true;
            }
            Some(x)
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::tokenizer::TokenType;

    use super::Tokenizer;

    fn tokenize_types<S: AsRef<str>>(s: S) -> Vec<TokenType> {
        let tokenizer = Tokenizer::new(s.as_ref());

        tokenizer.map(|v| v.unwrap().kind).collect::<Vec<_>>()
    }

    #[test]
    fn empty() {
        let tokens = tokenize_types("");

        assert_eq!(tokens, vec![TokenType::EOF]);
    }

    #[test]
    fn punctuation() {
        let tokens = tokenize_types("+ - * / // comment vs. slash");
        assert_eq!(
            tokens,
            vec![
                TokenType::Plus,
                TokenType::Minus,
                TokenType::Star,
                TokenType::Slash,
                TokenType::EOF
            ]
        );
    }

    #[test]
    fn strings() {
        let tokens = tokenize_types("\"hello world\"");
        assert_eq!(tokens, vec![TokenType::String, TokenType::EOF]);
    }
    #[test]
    fn numbers() {
        let tokens = tokenize_types("100.3");
        assert_eq!(tokens, vec![TokenType::Number, TokenType::EOF]);
    }
    #[test]
    fn identifiers() {
        let tokens = tokenize_types("hello_world");
        assert_eq!(tokens, vec![TokenType::Identifier, TokenType::EOF]);
    }
    #[test]
    fn keywords() {
        let tokens = tokenize_types("class if true");

        assert_eq!(
            tokens,
            vec![
                TokenType::Class,
                TokenType::If,
                TokenType::True,
                TokenType::EOF
            ]
        );
    }
}
