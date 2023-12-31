use std::collections::HashMap;
type Scope = HashMap<String, usize>;
use crate::assemble::encode;
use crate::bin::Instruction;
use crate::Error;

use super::scanner::Scanner;
use super::text::Token;
use super::text::TokenKind;
use super::vm::VM;
use super::Text;

pub(crate) struct Compiler<V: VM> {
    scanner: Scanner,
    vm: V,
    text: Text,
    token_buffer: Option<Token>,
    scopes: Vec<Scope>,
    offset: usize,
}

pub(crate) type CResult<T> = Result<T, Error>;

impl<V: VM> Compiler<V> {
    fn error_unexpected(&self, token: Token) -> Error {
        Error::UnexpectedToken(token)
    }
    fn error_immutable(&self, token: Token) -> Error {
        Error::Immutable(token)
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
        if op == "(" || op == "[" {
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
    fn token(&mut self) -> CResult<Token> {
        let t = self.scanner.next();
        if t.is_error() {
            Err(Error::Scanner)
        } else {
            Ok(t)
        }
    }
    fn pop(&mut self) -> CResult<Token> {
        if let Some(t) = self.token_buffer {
            self.token_buffer = None;
            Ok(t)
        } else {
            self.token()
        }
    }
    fn peek(&mut self) -> CResult<Token> {
        if let Some(t) = self.token_buffer {
            Ok(t)
        } else {
            let t = self.token()?;
            self.token_buffer = Some(t);
            Ok(t)
        }
    }
    fn emit(&mut self, instruction: Instruction) {
        let bytecode = encode(instruction);
        for i in 0..bytecode.len {
            self.vm.emit(bytecode.bytes[i as usize]);
        }
    }
    fn expect(&mut self, kind: TokenKind) -> CResult<Token> {
        let token = self.pop()?;
        if token.kind != kind {
            return Err(self.error_unexpected(token));
        }
        Ok(token)
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
    fn compile_atom(&mut self, token: Token) -> CResult<Instruction> {
        match token.kind {
            TokenKind::Number => Ok(Instruction::Konst(
                self.vm.rodata_number(
                    token
                        .text(self.text.clone())
                        .parse()
                        .expect("INVALID NUMERIC CONSTANT"),
                ),
            )),
            TokenKind::Literal => Ok({
                let name = token.text(self.text.clone());
                let name = name[1..name.len() - 1].to_string();
                Instruction::Konst(self.vm.rodata_literal(name))
            }),
            TokenKind::True => Ok(Instruction::True),
            TokenKind::False => Ok(Instruction::False),
            TokenKind::Nil => Ok(Instruction::Nil),
            TokenKind::Identifier => Ok(self.compile_load_id(token)?),
            _ => Err(self.error_unexpected(token)),
        }
    }
    fn expr(&mut self) -> CResult<()> {
        self.expr_p(0)
    }
    fn expr_p(&mut self, pwr: u32) -> CResult<()> {
        let token = self.pop()?;
        if let Some((_, rp)) = self.pwr_prefix(token.text(self.text.clone()).as_str()) {
            self.expr_p(rp)?;
            if token.kind == TokenKind::Single('-') {
                let address = self.vm.rodata_number(-1.0);
                self.emit(Instruction::Konst(address));
                self.emit(Instruction::Mult);
            }
        } else if token.text(self.text.clone()).as_str() == "(" {
            self.expr()?;
            self.expect(TokenKind::Single(')'))?;
        } else if token.text(self.text.clone()).as_str() == "[" {
            let count = self.explist(']')?;
            self.emit(Instruction::Anew(count));
        } else {
            let i = self.compile_atom(token)?;
            self.emit(i);
        }

        loop {
            let t = self.peek()?;
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
                self.pop()?;
                if t.kind == TokenKind::Single('(') {
                    let argc = self.explist(')')?;
                    self.emit(Instruction::Call(argc));
                } else {
                    self.expr()?;
                    self.expect(TokenKind::Single(']'))?;
                    self.emit(Instruction::Get);
                }
            } else if let Some((lp, rp)) = self.pwr_infix(ttext.as_str()) {
                if pwr > lp {
                    break;
                }
                self.pop()?;
                let i = self.compile_operator(t);
                self.expr_p(rp)?;
                self.emit(i);
            } else {
                return Err(self.error_unexpected(t));
            }
        }
        Ok(())
    }
    fn explist(&mut self, end: char) -> CResult<usize> {
        if self.peek()?.kind == TokenKind::Single(end) {
            self.pop()?;
            Ok(0)
        } else {
            let mut count = 0;
            loop {
                self.expr()?;
                count = count + 1;
                if self.peek()?.kind == TokenKind::Single(end) {
                    break;
                }
                self.expect(TokenKind::Single(','))?;
            }
            self.pop()?;
            Ok(count)
        }
    }
    pub(crate) fn new(text: Text, scanner: Scanner, vm: V) -> Compiler<V> {
        Compiler {
            scanner,
            vm,
            text,
            token_buffer: None,
            scopes: vec![Scope::default()],
            offset: 0,
        }
    }
    fn libs(&mut self) -> CResult<()> {
        // print
        let idx = self.vm.rodata_native(crate::native::bakht_print, 1);
        self.register_const("print".to_string(), idx)?;
        // len
        let idx = self.vm.rodata_native(crate::native::bakht_len, 1);
        self.register_const("len".to_string(), idx)?;
        // push
        let idx = self.vm.rodata_native(crate::native::bakht_push, 2);
        self.register_const("push".to_string(), idx)?;
        // pop
        let idx = self.vm.rodata_native(crate::native::bakht_pop, 1);
        self.register_const("pop".to_string(), idx)?;
        Ok(())
    }
    pub(crate) fn compile(&mut self) -> CResult<()> {
        self.libs()?;
        self.source()
    }
    pub(crate) fn vm(self) -> V {
        self.vm
    }
    fn flush_lvalue(&mut self, state: AssignCallState) -> CResult<()> {
        if let AssignCallState::Identifier(token) = state {
            let i = self.compile_load_id(token)?;
            self.emit(i);
        } else if state == AssignCallState::Index {
            self.emit(Instruction::Get);
        }
        Ok(())
    }
    fn get_token_text(&self, token: Token) -> String {
        token.text(self.text.clone())
    }
    fn get_id(&mut self, token: Token) -> CResult<(usize, bool)> {
        let name = self.get_token_text(token);
        for (i, c) in self.scopes.iter().enumerate().rev() {
            if let Some(idx) = c.get(&name) {
                return Ok((*idx, i == 0));
            }
        }
        return Err(Error::UnknownIdentifier(token));
    }
    fn compile_load_id(&mut self, token: Token) -> CResult<Instruction> {
        let (idx, is_global) = self.get_id(token)?;
        if is_global {
            Ok(Instruction::Konst(idx))
        } else {
            Ok(Instruction::Load(idx))
        }
    }
    fn compile_store_id(&mut self, token: Token) -> CResult<Instruction> {
        let (idx, is_global) = self.get_id(token)?;
        if is_global {
            Err(self.error_immutable(token))
        } else {
            Ok(Instruction::Store(idx))
        }
    }
    fn assign_call(&mut self) -> CResult<()> {
        let tkn = self.pop()?;
        let mut state = if tkn.is('(') {
            self.expr()?;
            self.expect(TokenKind::Single(')'))?;
            AssignCallState::InitialRvalue
        } else if tkn.is('[') {
            let count = self.explist(']')?;
            self.emit(Instruction::Anew(count));
            AssignCallState::InitialRvalue
        } else if tkn.kind == TokenKind::Identifier {
            AssignCallState::Identifier(tkn)
        } else {
            return Err(self.error_unexpected(tkn));
        };

        loop {
            let tkn = self.peek()?;
            if tkn.is('=') {
                self.pop()?;
                if state.endable() {
                    self.expr()?;
                    let i = if let AssignCallState::Identifier(token) = state {
                        self.compile_store_id(token)?
                    } else {
                        Instruction::Set
                    };
                    self.emit(i);
                    break;
                } else {
                    return Err(self.error_unexpected(tkn));
                }
            } else if tkn.is('[') {
                self.pop()?;
                self.flush_lvalue(state)?;
                self.expr()?;
                self.expect(TokenKind::Single(']'))?;
                state = AssignCallState::Index;
            } else if tkn.is('(') {
                self.pop()?;
                self.flush_lvalue(state)?;
                let count = self.explist(')')?;
                self.emit(Instruction::Call(count));
                state = AssignCallState::Call;
            } else {
                if state == AssignCallState::Call {
                    self.emit(Instruction::Pop(1));
                    break;
                } else {
                    return Err(self.error_unexpected(tkn));
                }
            }
        }
        Ok(())
    }
    fn block(&mut self, end: TokenKind) -> CResult<()> {
        self.new_scope();
        while self.peek()?.kind != end {
            self.stmt()?;
        }
        self.close_scope();
        self.pop()?;
        self.emit(Instruction::Nil);
        self.emit(Instruction::Ret);
        Ok(())
    }
    fn new_scope(&mut self) {
        self.scopes.push(Scope::default());
    }
    fn close_scope(&mut self) {
        let scope_size = self.curscope().len();
        self.offset -= scope_size;
        if scope_size > 0 {
            self.emit(Instruction::Pop(scope_size));
        }
        self.scopes.pop();
    }
    fn curscope<'a>(&'a mut self) -> &'a mut Scope {
        self.scopes.last_mut().unwrap()
    }
    fn register_decl(&mut self, token: Token) -> CResult<()> {
        let name = self.get_token_text(token);
        if self.curscope().get(&name).is_some() {
            return Err(Error::MultipleDefinition(name));
        }
        let idx = self.offset;
        self.offset += 1;
        self.curscope().insert(name, idx);
        Ok(())
    }
    fn register_const(&mut self, name: String, idx: usize) -> CResult<()> {
        if self.scopes.first().unwrap().get(&name).is_some() {
            return Err(Error::MultipleDefinition(name));
        }
        self.curscope().insert(name, idx);
        Ok(())
    }
    fn var_decl(&mut self) -> CResult<()> {
        let id = self.expect(TokenKind::Identifier)?;
        self.register_decl(id)?;
        if self.peek()?.is('=') {
            self.pop()?;
            self.expr()?;
        } else {
            self.emit(Instruction::Nil);
        }
        Ok(())
    }
    fn stmt(&mut self) -> CResult<()> {
        if self.peek()?.is('{') {
            self.pop()?;
            self.block(TokenKind::Single('}'))?;
        } else if self.peek()?.kind == TokenKind::Let {
            self.pop()?;
            self.var_decl()?;
            while self.peek()?.is(',') {
                self.pop()?;
                self.var_decl()?;
            }
        } else if self.peek()?.kind == TokenKind::Return {
            self.pop()?;
            self.expr()?;
            self.emit(Instruction::Ret);
            while !self.peek()?.is('}') && self.peek()?.kind != TokenKind::EOF {
                self.pop()?;
            }
        } else if self.peek()?.kind == TokenKind::If {
            self.if_stmt()?;
        } else {
            self.assign_call()?;
        }
        Ok(())
    }
    fn seek(&mut self, kind: TokenKind) -> CResult<bool> {
        Ok(true)
    }
    fn if_stmt(&mut self) -> CResult<()> {
        self.pop()?;
        self.expr()?;
        self.expect(TokenKind::Single('{'))?;
        self.new_scope();
        while self.peek()?.kind != TokenKind::Single('}') || self.peek()?.kind != TokenKind::EOF {}
        self.expect(TokenKind::Single('}'))?;
        Ok(())
    }
    fn paramlist(&mut self) -> CResult<u8> {
        self.expect(TokenKind::Single('('))?;
        if self.peek()?.is(')') {
            self.pop()?;
            Ok(0)
        } else {
            let id = self.expect(TokenKind::Identifier)?;
            self.register_decl(id)?;
            let mut param_count = 1;
            while self.peek()?.is(',') {
                self.pop()?;
                let id = self.expect(TokenKind::Identifier)?;
                self.register_decl(id)?;
                param_count = param_count + 1;
            }
            self.expect(TokenKind::Single(')'))?;
            Ok(param_count)
        }
    }
    fn function_body(&mut self) -> CResult<bool> {
        let id = self.expect(TokenKind::Identifier)?;
        let param_count = self.paramlist()? as usize;
        let is_main = self.get_token_text(id).as_str() == "main";
        let idx = self.vm.rodata_function(param_count, is_main);
        self.register_const(self.get_token_text(id), idx)?;
        self.expect(TokenKind::Single('{'))?;
        self.new_scope();
        self.block(TokenKind::Single('}'))?;
        self.close_scope();
        Ok(is_main)
    }
    fn source(&mut self) -> CResult<()> {
        let mut has_main = false;
        while self.peek()?.kind != TokenKind::EOF {
            let token = self.pop()?;
            if token.kind == TokenKind::Fn {
                has_main |= self.function_body()?;
            } else {
                return Err(self.error_unexpected(token));
            }
        }
        self.pop()?;
        if has_main {
            Ok(())
        } else {
            Err(Error::NoMainFunction)
        }
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
