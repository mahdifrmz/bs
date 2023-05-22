use super::Text;
use crate::text::{Token, TokenKind};

const SINGLE_CHARS: &[char] = &[
    '+', '-', '*', '/', '%', '[', ']', '(', ')', '{', '}', ',', '.',
];
const EQUAL_FOLLOW: &[char] = &['=', '>', '<', '!'];

pub(crate) struct Scanner {
    pub(crate) text: Text,
    pub(crate) old_ptr: usize,
    pub(crate) ptr: usize,
}

impl Scanner {
    pub fn new(text: Text) -> Scanner {
        Scanner {
            text,
            ptr: 0,
            old_ptr: 0,
        }
    }
    fn token(&mut self, kind: TokenKind) -> Token {
        Token {
            from: self.old_ptr,
            len: self.ptr - self.old_ptr,
            kind,
        }
    }
    fn peek(&mut self) -> char {
        if self.ptr >= self.text.len() {
            '\0'
        } else {
            self.text[self.ptr]
        }
    }
    fn pop(&mut self) -> char {
        let c = self.peek();
        self.ptr = self.ptr + 1;
        c
    }
    fn read(&mut self) -> Token {
        let c = self.peek();
        if c == '\0' {
            return self.token(TokenKind::EOF);
        }
        self.pop();
        if c.is_whitespace() {
            while self.peek().is_whitespace() {
                self.pop();
            }
            self.token(TokenKind::White)
        } else if c.is_ascii_alphabetic() || c == '_' {
            while self.peek().is_ascii_alphanumeric() || c == '_' {
                self.pop();
            }
            let mut token = self.token(TokenKind::Identifier);
            let tt = token.text(self.text.clone());
            if tt.as_str() == "if" {
                token.kind = TokenKind::If;
            } else if tt.as_str() == "while" {
                token.kind = TokenKind::While;
            } else if tt.as_str() == "else" {
                token.kind = TokenKind::Else;
            } else if tt.as_str() == "fn" {
                token.kind = TokenKind::Fn;
            } else if tt.as_str() == "false" {
                token.kind = TokenKind::False;
            } else if tt.as_str() == "true" {
                token.kind = TokenKind::True;
            } else if tt.as_str() == "nil" {
                token.kind = TokenKind::Nil;
            } else if tt.as_str() == "return" {
                token.kind = TokenKind::Return;
            } else if tt.as_str() == "let" {
                token.kind = TokenKind::Let;
            }
            token
        } else if c == '\'' {
            while self.peek() != '\'' && self.peek() != '\0' {
                self.pop();
            }
            if self.peek() == '\'' {
                self.pop();
                self.token(TokenKind::Literal)
            } else {
                self.token(TokenKind::Error)
            }
        } else if c.is_ascii_digit() {
            while self.peek().is_ascii_digit() {
                self.pop();
            }
            self.token(TokenKind::Number)
        } else if c == '#' {
            while self.peek() != '\n' && self.peek() != '\0' {
                self.pop();
            }
            if self.peek() == '\n' {
                self.pop();
            }
            self.token(TokenKind::Comment)
        } else if SINGLE_CHARS.contains(&c) {
            self.token(TokenKind::Single(c))
        } else if EQUAL_FOLLOW.contains(&c) {
            if self.peek() == '=' {
                self.pop();
                self.token(TokenKind::Double)
            } else {
                self.token(TokenKind::Single(c))
            }
        } else {
            self.token(TokenKind::Error)
        }
    }
    fn sync(&mut self) {
        self.old_ptr = self.ptr
    }
    pub fn next(&mut self) -> Token {
        let mut t = self.read();
        self.sync();
        while t.is_discardable() {
            t = self.read();
            self.sync();
        }
        t
    }
}
