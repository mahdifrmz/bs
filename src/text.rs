use std::sync::Arc;

pub(crate) type Text = Arc<Vec<char>>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TokenKind {
    White,
    Comment,
    Identifier,
    Number,
    Single(char),
    Double,
    Error,
    EOF,
    Literal,
    // keywords
    Let,
    If,
    Else,
    While,
    Fn,
    Nil,
    True,
    False,
    Return,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Token {
    pub(crate) from: usize,
    pub(crate) len: usize,
    pub(crate) kind: TokenKind,
}

impl Token {
    pub(crate) fn is_discardable(&self) -> bool {
        self.kind == TokenKind::White || self.kind == TokenKind::Comment
    }
    pub(crate) fn is_error(&self) -> bool {
        self.kind == TokenKind::Error
    }
    pub(crate) fn is(&self, c: char) -> bool {
        self.kind == TokenKind::Single(c)
    }
    pub(crate) fn text(&self, text: Text) -> String {
        String::from_iter(text[self.from..self.from + self.len].iter())
    }
}
