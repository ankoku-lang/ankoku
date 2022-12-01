#[derive(Clone, Debug, PartialEq, Eq)]
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
    String(String),
    Number(String),
    Identifier(String),
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
    Error(String),
    EOF,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    source: Vec<char>,
    start: usize,
    current: usize,
    line: u32,
    done: bool,
}
impl Tokenizer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            start: 0,
            done: false,
        }
    }
    pub fn at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;
        if self.at_end() {
            return self.new_token(TokenType::EOF);
        }
        let c = self.advance();
        let eqm = self.mtch('=');
        println!("{}", c);
        match c {
            '(' => return self.new_token(TokenType::LParen),
            ')' => return self.new_token(TokenType::RParen),
            '{' => return self.new_token(TokenType::LBrace),
            '}' => return self.new_token(TokenType::RBrace),
            ';' => return self.new_token(TokenType::Semicolon),
            ',' => return self.new_token(TokenType::Dot),
            '-' => return self.new_token(TokenType::Minus),
            '+' => return self.new_token(TokenType::Plus),
            '/' => {
                println!("slash");
                return self.new_token(TokenType::Slash);
            }
            '*' => return self.new_token(TokenType::Star),
            '!' => {
                return self.new_token(if eqm {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                })
            }
            '=' => {
                return self.new_token(if eqm {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                })
            }
            '<' => {
                return self.new_token(if eqm {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                })
            }
            '>' => {
                return self.new_token(if eqm {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                })
            }
            '"' => {
                return self.string();
            }
            'a'..='z' | 'A'..='Z' => {}
            '0'..='9' => {
                return self.number();
            }
            _ => {}
        }

        self.error_token("Unexpected character.")
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
        return self.new_token(TokenType::Number(
            self.source[self.start..=self.current - 1].iter().collect(),
        ));
    }
    fn string(&mut self) -> Token {
        while self.peek() != Some('"') && !self.at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }
        if self.at_end() {
            return self.error_token("Unterminated string.");
        }
        self.advance();
        return self.new_token(TokenType::String(
            self.source[self.start + 1..=self.current - 2]
                .iter()
                .collect(),
        ));
    }
    fn skip_whitespace(&mut self) {
        while self.peek().map_or(false, |v| v.is_whitespace() || v == '/') {
            if self.peek().unwrap() == '\n' {
                self.line += 1;
            }
            if self.peek().unwrap() == '/' && self.peek_next() == Some('/') {
                println!("comment {}", self.current);
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
    fn error_token<A: Into<String>>(&self, msg: A) -> Token {
        Token {
            kind: TokenType::Error(msg.into()),
            start: self.start,
            length: 0,
            line: self.line,
        }
    }
}
impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let x = self.next_token();
            if x.kind == TokenType::EOF || matches!(x.kind, TokenType::Error(_)) {
                self.done = true
            }
            Some(x)
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::tokenizer::TokenType;

    use super::Tokenizer;

    #[test]
    fn empty() {
        let tokenizer = Tokenizer::new("");
        let tokens = tokenizer.map(|v| v.kind).collect::<Vec<_>>();

        assert_eq!(tokens, vec![TokenType::EOF]);
    }

    #[test]
    fn punctuation() {
        let tokenizer = Tokenizer::new("+ - * / // comment vs. slash");
        let tokens = tokenizer.map(|v| v.kind).collect::<Vec<_>>();
        println!("{:?}", tokens);
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
        let tokenizer = Tokenizer::new("\"hello world\"");
        let tokens = tokenizer.map(|v| v.kind).collect::<Vec<_>>();
        println!("{:?}", tokens);
        assert_eq!(
            tokens,
            vec![TokenType::String("hello world".into()), TokenType::EOF]
        );
    }
    #[test]
    fn numbers() {
        let tokenizer = Tokenizer::new("1000");
        let tokens = tokenizer.map(|v| v.kind).collect::<Vec<_>>();
        println!("{:?}", tokens);
        assert_eq!(
            tokens,
            vec![TokenType::Number("1000".into()), TokenType::EOF]
        );
    }
}
