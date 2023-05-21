use std::{collections::HashMap, process::exit, sync::Arc};

type Text = Arc<Vec<char>>;

const SINGLE_CHARS: &[char] = &[
    '+', '-', '*', '/', '%', '[', ']', '(', ')', '{', '}', ',', '.',
];
const EQUAL_FOLLOW: &[char] = &['=', '>', '<', '!'];

struct Scanner {
    text: Text,
    old_ptr: usize,
    ptr: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TokenKind {
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

#[derive(Clone, Copy, PartialEq, Eq)]
struct Token {
    from: usize,
    len: usize,
    kind: TokenKind,
}

impl Token {
    fn is_discardable(&self) -> bool {
        self.kind == TokenKind::White || self.kind == TokenKind::Comment
    }
    fn is_error(&self) -> bool {
        self.kind == TokenKind::Error
    }
    fn is(&self, c: char) -> bool {
        self.kind == TokenKind::Single(c)
    }
    fn text(&self, text: Text) -> String {
        String::from_iter(text[self.from..self.from + self.len].iter())
    }
}

impl Scanner {
    fn new(text: Text) -> Scanner {
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
    fn next(&mut self) -> Token {
        let mut t = self.read();
        self.sync();
        while t.is_discardable() {
            t = self.read();
            self.sync();
        }
        t
    }
}

type Native = fn(&mut BakhtScript);

enum Function {
    Bakht { param_count: u8, address: usize },
    Native(Native),
}

enum Value {
    String(String),
    Array(Vec<Value>),
    Nil,
    Boolean(bool),
    Number(f32),
    Function(Function),
}

trait VM {
    fn function(&mut self, param_count: u8);
    fn emit(&mut self, bytecode: u8);
    fn rodata_number(&mut self, number: f32) -> usize;
    fn rodata_literal(&mut self, literal: String) -> usize;
    fn run(&mut self) {}
}

#[derive(Default)]
struct BVM {}

impl VM for BVM {
    // FIXME
    fn emit(&mut self, bytecode: u8) {
        print!("{}:", bytecode)
    }
    fn rodata_number(&mut self, number: f32) -> usize {
        0
    }
    fn rodata_literal(&mut self, literal: String) -> usize {
        0
    }
    fn run(&mut self) {}
    fn function(&mut self, param_count: u8) {}
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Instruction {
    Add = 0,
    Sub = 1,
    Mult = 2,
    Div = 3,
    Eq = 4,
    Ne = 5,
    Ge = 6,
    Le = 7,
    Gt = 8,
    Lt = 9,
    Set = 10,
    Get = 11,
    Pop = 12,
    Ret = 13,
    Load(usize) = 14,
    Store(usize) = 15,
    Call(usize) = 16,
    Konst(usize) = 17,
    Nil = 18,
    True = 19,
    False = 20,
    NewArray(usize) = 21,
    Mod = 22,
    GLoad(usize) = 23,
    GStore(usize) = 24,
}
impl Instruction {
    fn encode_params(self) -> (u8, Option<usize>) {
        match self {
            Instruction::Add => (0, None),
            Instruction::Sub => (1, None),
            Instruction::Mult => (2, None),
            Instruction::Div => (3, None),
            Instruction::Eq => (4, None),
            Instruction::Ne => (5, None),
            Instruction::Ge => (6, None),
            Instruction::Le => (7, None),
            Instruction::Gt => (8, None),
            Instruction::Lt => (9, None),
            Instruction::Set => (10, None),
            Instruction::Get => (11, None),
            Instruction::Pop => (12, None),
            Instruction::Ret => (13, None),
            Instruction::Load(o) => (14, Some(o)),
            Instruction::Store(o) => (15, Some(o)),
            Instruction::Call(o) => (16, Some(o)),
            Instruction::Konst(o) => (17, Some(o)),
            Instruction::Nil => (18, None),
            Instruction::True => (19, None),
            Instruction::False => (20, None),
            Instruction::NewArray(o) => (21, Some(o)),
            Instruction::Mod => (22, None),
            Instruction::GLoad(o) => (23, Some(o)),
            Instruction::GStore(o) => (24, Some(o)),
        }
    }
}

type Scope = HashMap<String, usize>;

struct Compiler<V: VM> {
    scanner: Scanner,
    vm: V,
    text: Text,
    token_buffer: Option<Token>,
    scopes: Vec<Scope>,
    offset: usize,
}

impl<V: VM> Compiler<V> {
    fn error_unexpected(&self, token: Token) -> ! {
        let token_text = if token.kind == TokenKind::EOF {
            "EOF".to_string()
        } else {
            token.text(self.text.clone())
        };
        eprintln!(
            "Bakht Error: Unexpected token {} at {}",
            token_text, token.from
        );
        exit(1);
    }
    fn pwr_infix(&self, op: &str) -> Option<(u32, u32)> {
        if op == "+" || op == "-" {
            Some((51, 52))
        } else if op == "*" || op == "/" || op == "%" {
            Some((53, 54))
        } else if op == "<" || op == ">" || op == "<=" || op == ">=" || op == "==" || op == "!=" {
            Some((49, 50))
        } else {
            None
        }
    }
    fn pwr_postfix(&self, op: &str) -> Option<(u32, ())> {
        if op == "(" || op == "[" || op == "." {
            Some((59, ()))
        } else {
            None
        }
    }
    fn pwr_prefix(&self, op: &str) -> Option<((), u32)> {
        if op == "+" || op == "-" {
            Some(((), 56))
        } else {
            None
        }
    }
    fn token(&mut self) -> Token {
        let t = self.scanner.next();
        if t.is_error() {
            panic!("SCANNER ERROR");
        }
        t
    }
    fn pop(&mut self) -> Token {
        if let Some(t) = self.token_buffer {
            self.token_buffer = None;
            t
        } else {
            self.token()
        }
    }
    fn peek(&mut self) -> Token {
        if let Some(t) = self.token_buffer {
            t
        } else {
            let t = self.token();
            self.token_buffer = Some(t);
            t
        }
    }
    fn encode(&self, instruction: Instruction) -> Bytecode {
        let i = instruction.encode_params();
        let opcode = i.0 as u8;
        let operand = i.1 as Option<usize>;
        let mut bytecode = Bytecode::default();
        bytecode.bytes[0] = opcode;
        bytecode.len = 1;
        if let Some(mut o) = operand {
            if o > 0xffffffff {
                bytecode.bytes[0] = bytecode.bytes[0] | 0b_1100_0000;
                bytecode.len = 9;
                for i in 1..9 {
                    bytecode.bytes[i] = (o & 0xff) as u8;
                    o = o >> 8;
                }
            } else if o > 0xffff {
                bytecode.bytes[0] = bytecode.bytes[0] | 0b_1000_0000;
                bytecode.len = 5;
                for i in 1..5 {
                    bytecode.bytes[i] = (o & 0xff) as u8;
                    o = o >> 8;
                }
            } else if o > 0xff {
                bytecode.bytes[0] = bytecode.bytes[0] | 0b_0100_0000;
                bytecode.len = 3;
                for i in 1..3 {
                    bytecode.bytes[i] = (o & 0xff) as u8;
                    o = o >> 8;
                }
            } else {
                bytecode.bytes[0] = bytecode.bytes[0];
                bytecode.len = 2;
                bytecode.bytes[1] = o as u8;
            }
        }
        bytecode
    }
    fn emit(&mut self, instruction: Instruction) {
        let bytecode = self.encode(instruction);
        for i in 0..bytecode.len {
            self.vm.emit(bytecode.bytes[i as usize])
        }
    }
    fn expect(&mut self, kind: TokenKind) -> Token {
        let token = self.pop();
        if token.kind != kind {
            self.error_unexpected(token);
        }
        token
    }
    fn compile_operator(&mut self, token: Token) -> Instruction {
        match token.kind {
            TokenKind::Single('+') => Instruction::Add,
            TokenKind::Single('-') => Instruction::Sub,
            TokenKind::Single('*') => Instruction::Mult,
            TokenKind::Single('/') => Instruction::Div,
            TokenKind::Single('%') => Instruction::Mod,
            TokenKind::Double => match token.text(self.text.clone()).as_str() {
                "==" => Instruction::Eq,
                "!=" => Instruction::Ne,
                ">=" => Instruction::Ge,
                "<=" => Instruction::Le,
                "<" => Instruction::Lt,
                ">" => Instruction::Gt,
                _ => panic!("IMPOSSIBLE!"),
            },
            _ => panic!("IMPOSSIBLE!"),
        }
    }
    fn compile_atom(&mut self, token: Token) -> Instruction {
        match token.kind {
            TokenKind::Number => Instruction::Konst(
                self.vm.rodata_number(
                    token
                        .text(self.text.clone())
                        .parse()
                        .expect("INVALID NUMERIC CONSTANT"),
                ),
            ),
            TokenKind::Literal => {
                Instruction::Konst(self.vm.rodata_literal(token.text(self.text.clone())))
            }
            TokenKind::True => Instruction::True,
            TokenKind::False => Instruction::False,
            TokenKind::Nil => Instruction::Nil,
            TokenKind::Identifier => self.compile_load_id(token),
            _ => self.error_unexpected(token),
        }
    }
    fn expr(&mut self) {
        self.expr_p(0)
    }
    fn property(&mut self) {
        let id = self.expect(TokenKind::Identifier);
        let prop = self.vm.rodata_literal(id.text(self.text.clone()));
        self.emit(Instruction::Konst(prop));
    }
    fn expr_p(&mut self, pwr: u32) {
        let token = self.pop();
        if let Some((_, rp)) = self.pwr_prefix(token.text(self.text.clone()).as_str()) {
            self.expr_p(rp);
            if token.kind == TokenKind::Single('-') {
                let address = self.vm.rodata_number(-1.0);
                self.emit(Instruction::Konst(address));
                self.emit(Instruction::Mult);
            }
        } else if token.text(self.text.clone()).as_str() == "(" {
            self.expr();
            self.expect(TokenKind::Single(')'));
        } else if token.text(self.text.clone()).as_str() == "[" {
            let count = self.explist(']');
            self.emit(Instruction::NewArray(count));
        } else {
            let i = self.compile_atom(token);
            self.emit(i);
        }

        loop {
            let t = self.peek();
            match t.kind {
                TokenKind::Single(c) => {
                    if c == '}' || c == '{' || c == ',' || c == ')' || c == ']' {
                        break;
                    } else {
                        ()
                    }
                }
                TokenKind::Double => (),
                _ => break,
            }
            let ttext = t.text(self.text.clone());
            if let Some((lp, _)) = self.pwr_postfix(ttext.as_str()) {
                if token.kind != TokenKind::Identifier && !token.is('(') && !token.is('[') {
                    break;
                }
                if pwr > lp {
                    break;
                }
                self.pop();
                if t.kind == TokenKind::Single('(') {
                    let argc = self.explist(')');
                    self.emit(Instruction::Call(argc));
                } else if t.kind == TokenKind::Single('.') {
                    self.property();
                    self.emit(Instruction::Get)
                } else {
                    self.expr();
                    self.expect(TokenKind::Single(']'));
                    self.emit(Instruction::Set);
                }
            } else if let Some((lp, rp)) = self.pwr_infix(ttext.as_str()) {
                if pwr > lp {
                    break;
                }
                self.pop();
                let i = self.compile_operator(t);
                self.expr_p(rp);
                self.emit(i);
            } else {
                self.error_unexpected(t);
            }
        }
    }
    fn explist(&mut self, end: char) -> usize {
        if self.peek().kind == TokenKind::Single(end) {
            self.pop();
            0
        } else {
            let mut count = 0;
            loop {
                self.expr();
                count = count + 1;
                if self.peek().kind == TokenKind::Single(end) {
                    break;
                }
                self.expect(TokenKind::Single(','));
            }
            self.pop();
            count
        }
    }
    fn new(text: Text, scanner: Scanner, vm: V) -> Compiler<V> {
        Compiler {
            scanner,
            vm,
            text,
            token_buffer: None,
            scopes: vec![Scope::default()],
            offset: 0,
        }
    }
    fn compile(&mut self) {
        self.source()
    }
    fn vm(self) -> V {
        self.vm
    }
    fn flush_lvalue(&mut self, state: AssignCallState) {
        if let AssignCallState::Identifier(token) = state {
            let i = self.compile_load_id(token);
            self.emit(i);
        } else if state == AssignCallState::Index {
            self.emit(Instruction::Get);
        }
    }
    fn get_token_text(&self, token: Token) -> String {
        token.text(self.text.clone())
    }
    fn get_id(&mut self, token: Token) -> (usize, bool) {
        let name = self.get_token_text(token);
        for (i, c) in self.scopes.iter().enumerate().rev() {
            if let Some(idx) = c.get(&name) {
                return (*idx, i == 0);
            }
        }
        eprintln!("unknown identifier '{}' at {}", name, token.from);
        exit(1);
    }
    fn compile_load_id(&mut self, token: Token) -> Instruction {
        let (idx, is_global) = self.get_id(token);
        if is_global {
            Instruction::GLoad(idx)
        } else {
            Instruction::Load(idx)
        }
    }
    fn compile_store_id(&mut self, token: Token) -> Instruction {
        let (idx, is_global) = self.get_id(token);
        if is_global {
            Instruction::GStore(idx)
        } else {
            Instruction::Store(idx)
        }
    }
    fn assign_call(&mut self) {
        let tkn = self.pop();
        let mut state = if tkn.is('(') {
            self.expr();
            self.expect(TokenKind::Single(')'));
            AssignCallState::InitialRvalue
        } else if tkn.is('[') {
            let count = self.explist(']');
            self.emit(Instruction::NewArray(count));
            AssignCallState::InitialRvalue
        } else if tkn.kind == TokenKind::Identifier {
            AssignCallState::Identifier(tkn)
        } else {
            self.error_unexpected(tkn);
        };

        loop {
            let tkn = self.peek();
            if tkn.is('=') {
                self.pop();
                if state.endable() {
                    self.expr();
                    let i = if let AssignCallState::Identifier(token) = state {
                        self.compile_store_id(token)
                    } else {
                        Instruction::Set
                    };
                    self.emit(i);
                    break;
                } else {
                    self.error_unexpected(tkn);
                }
            } else if tkn.is('[') {
                self.pop();
                self.flush_lvalue(state);
                self.expr();
                self.expect(TokenKind::Single(']'));
                state = AssignCallState::Index;
            } else if tkn.is('.') {
                self.pop();
                self.flush_lvalue(state);
                self.property();
                state = AssignCallState::Index;
            } else if tkn.is('(') {
                self.pop();
                self.flush_lvalue(state);
                let count = self.explist(')');
                self.emit(Instruction::Call(count));
                state = AssignCallState::Call;
            } else {
                if state == AssignCallState::Call {
                    break;
                } else {
                    self.error_unexpected(tkn);
                }
            }
        }
    }
    fn block(&mut self, end: TokenKind) {
        self.new_scope();
        while self.peek().kind != end {
            self.stmt();
        }
        self.close_scope();
        self.expect(end);
    }
    fn new_scope(&mut self) {
        self.scopes.push(Scope::default());
    }
    fn close_scope(&mut self) {
        self.offset -= self.curscope().len();
        self.scopes.pop();
    }
    fn curscope<'a>(&'a mut self) -> &'a mut Scope {
        self.scopes.last_mut().unwrap()
    }
    fn register_decl(&mut self, token: Token) {
        let name = self.get_token_text(token);
        if self.curscope().get(&name).is_some() {
            eprintln!("Variable '{}' previously defined", name);
            exit(1);
        }
        let idx = self.offset;
        self.offset += 1;
        self.curscope().insert(name, idx);
    }
    fn var_decl(&mut self) {
        let id = self.expect(TokenKind::Identifier);
        self.register_decl(id);
        if self.peek().is('=') {
            self.pop();
            self.expr();
        } else {
            self.emit(Instruction::Nil)
        }
    }
    fn stmt(&mut self) {
        if self.peek().is('{') {
            self.block(TokenKind::Single('}'));
        } else if self.peek().kind == TokenKind::Let {
            self.pop();
            self.var_decl();
            while self.peek().is(',') {
                self.pop();
                self.var_decl();
            }
        } else {
            self.assign_call();
        }
    }
    fn paramlist(&mut self) -> u8 {
        self.expect(TokenKind::Single('('));
        if self.peek().is(')') {
            self.pop();
            0
        } else {
            let id = self.expect(TokenKind::Identifier);
            self.register_decl(id);
            let mut param_count = 1;
            while self.peek().is(',') {
                self.pop();
                let id = self.expect(TokenKind::Identifier);
                self.register_decl(id);
                param_count = param_count + 1;
            }
            self.expect(TokenKind::Single(')'));
            param_count
        }
    }
    fn function_body(&mut self) {
        let id = self.expect(TokenKind::Identifier);
        self.register_decl(id);
        self.new_scope();
        let param_count = self.paramlist();
        self.vm.function(param_count);
        self.expect(TokenKind::Single('{'));
        self.block(TokenKind::Single('}'));
        self.close_scope();
    }
    fn source(&mut self) {
        while self.peek().kind != TokenKind::EOF {
            let token = self.pop();
            if token.kind == TokenKind::Fn {
                self.function_body();
            } else {
                self.error_unexpected(token);
            }
        }
        self.pop();
    }
}

#[derive(PartialEq, Eq)]
enum AssignCallState {
    InitialRvalue,
    Call,
    Identifier(Token),
    Index,
}

impl AssignCallState {
    fn endable(&self) -> bool {
        match self {
            AssignCallState::InitialRvalue | AssignCallState::Call => false,
            AssignCallState::Identifier(_) | AssignCallState::Index => true,
        }
    }
}

#[derive(Default)]
struct Bytecode {
    bytes: [u8; 9],
    len: u8,
}

#[derive(Default)]
struct BakhtScript {}

impl BakhtScript {
    fn run(&self, source: &str) {
        let text: Text = Arc::new(source.chars().collect());
        let scanner = Scanner::new(text.clone());
        let vm = BVM {};
        let mut compiler = Compiler::new(text, scanner, vm);
        compiler.compile();
        let mut vm = compiler.vm();
        vm.run();
    }
}

fn main() {
    let bs = BakhtScript::default();
    bs.run(
        std::fs::read_to_string("./local/source.bs")
            .unwrap()
            .as_str(),
    );
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{Instruction, Scanner};

    use super::{Compiler, VM};

    #[derive(Default)]
    struct MockVM {
        bin: Vec<u8>,
        cidx: usize,
    }

    impl VM for MockVM {
        fn run(&mut self) {}
        fn function(&mut self, param_count: u8) {}

        fn emit(&mut self, bytecode: u8) {
            self.bin.push(bytecode)
        }

        fn rodata_number(&mut self, _: f32) -> usize {
            let cidx = self.cidx;
            self.cidx = self.cidx + 1;
            cidx
        }

        fn rodata_literal(&mut self, _: String) -> usize {
            let cidx = self.cidx;
            self.cidx = self.cidx + 1;
            cidx
        }
    }

    impl MockVM {
        fn check(&self, check: &[u8]) {
            assert!(check == self.bin);
        }
    }

    fn check(src: &str, target: &[Instruction]) {
        let target = target
            .iter()
            .map(|i| i.encode_params())
            .map(|(i, o)| match o {
                Some(v) => vec![i, v as u8],
                None => vec![i],
            })
            .flatten()
            .collect::<Vec<_>>();
        let text: Arc<Vec<char>> = Arc::new(src.chars().collect());
        let vm = MockVM::default();
        let scanner = Scanner::new(text.clone());
        let mut compiler = Compiler::new(text, scanner, vm);
        compiler.compile();
        let vm = compiler.vm();
        vm.check(target.as_slice());
    }

    #[test]
    fn empty() {
        check("", &[]);
    }

    #[test]
    fn number() {
        check("let a = 3", &[Instruction::Konst(0)]);
    }

    #[test]
    fn variable() {
        check("let a = b", &[Instruction::Load(0)]);
    }

    #[test]
    fn operator() {
        check(
            "let a = b + 1",
            &[
                Instruction::Load(0),
                Instruction::Konst(0),
                Instruction::Add,
            ],
        );
    }

    #[test]
    fn multi_operator() {
        check(
            "let a = b + 1 * 5",
            &[
                Instruction::Load(0),
                Instruction::Konst(0),
                Instruction::Konst(1),
                Instruction::Mult,
                Instruction::Add,
            ],
        );
    }

    #[test]
    fn parentheses() {
        check(
            "let a = (b + 1)",
            &[
                Instruction::Load(0),
                Instruction::Konst(0),
                Instruction::Add,
            ],
        );
    }
}
