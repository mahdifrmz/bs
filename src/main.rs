use std::{collections::HashMap, sync::Arc};

type Text = Arc<Vec<char>>;

const single_chars: &[char] = &['+', '-', '*', '/', '[', ']', '(', ')', '{', '}', ','];
const equal_follow: &[char] = &['=', '>', '<', '!'];

struct Lexer {
    text: Text,
    old_ptr: usize,
    ptr: usize,
}

#[derive(PartialEq, Eq)]
enum TokenKind {
    White,
    Comment,
    Identifier,
    Number,
    Single,
    Double,
    Error,
    EOF,
    Literal,
    // keywords
    If,
    Else,
    While,
    Fn,
    Nil,
    True,
    False,
    Return,
}

struct Token {
    from: usize,
    len: usize,
    kind: TokenKind,
}

impl Token {
    fn is_discardable(&self) -> bool {
        return self.kind == TokenKind::White || self.kind == TokenKind::Comment;
    }
    fn is_error(&self) -> bool {
        return self.kind == TokenKind::Error;
    }
    fn text(&self, text: Text) -> String {
        return String::from_iter(text[self.from..self.from + self.len].iter());
    }
}

impl Lexer {
    fn new(text: Text) -> Lexer {
        Lexer {
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
        self.text[self.ptr]
    }
    fn pop(&mut self) -> char {
        let c = self.peek();
        self.ptr = self.ptr + 1;
        c
    }
    fn read(&mut self) -> Token {
        if self.ptr >= self.text.len() {
            return self.token(TokenKind::EOF);
        }
        let c = self.pop();
        if c.is_whitespace() {
            while self.peek().is_whitespace() {
                self.pop();
            }
            self.token(TokenKind::White)
        } else if c.is_ascii_alphabetic() || c == '_' {
            while self.peek().is_ascii_alphanumeric() || c == '_' {
                self.pop();
            }
            self.token(TokenKind::Identifier)
        } else if c == '\'' {
            while self.peek() != '\'' {
                self.pop();
            }
            self.pop();
            self.token(TokenKind::Literal)
        } else if c.is_ascii_digit() {
            while self.peek().is_ascii_digit() {
                self.pop();
            }
            let mut token = self.token(TokenKind::Number);
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
            }
            token
        } else if c == '#' {
            while self.peek() != '\n' {
                self.pop();
            }
            self.pop();
            self.token(TokenKind::Comment)
        } else if single_chars.contains(&c) {
            self.token(TokenKind::Single)
        } else if equal_follow.contains(&c) {
            if self.peek() == '=' {
                self.pop();
                self.token(TokenKind::Double)
            } else {
                self.token(TokenKind::Single)
            }
        } else {
            self.token(TokenKind::Error)
        }
    }
    fn next(&mut self) -> Token {
        let mut t = self.read();
        while t.is_discardable() {
            t = self.read()
        }
        t
    }
}

enum Value {
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Nil,
    Boolean(bool),
    Number(f32),
    Function(u32),
}

struct VM {}

impl VM {
    fn emit(bytecode: u8) {}
    fn rodata(bytecode: u8) {}
}

struct Compiler {
    lexer: Lexer,
    vm: VM,
}

impl Compiler {
    fn expr(pwr: u32) {}
}

fn main() {
    println!("Hello, world!");
}
