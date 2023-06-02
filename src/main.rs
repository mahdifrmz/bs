mod assemble;
mod bin;
mod compiler;
mod native;
mod scanner;
#[cfg(test)]
mod tests;
mod text;
mod vm;

use compiler::CResult;
use scanner::Scanner;
use std::sync::Arc;
use text::{Text, Token};
use vm::{Value, BVM};

#[derive(Debug, Clone)]
pub(crate) enum Error {
    Scanner,
    UnexpectedToken(Token),
    Immutable(Token),
    NoMainFunction,
    InvalidOperands,
    IndexOutOfBound,
    DivisionByZero,
    CallingNonFunction,
    UnknownIdentifier(Token),
    MultipleDefinition(String),
}

#[derive(Default)]
struct BakhtScript {
    vm: BVM,
}

enum BakhtValue {
    Function,
    Boolean(bool),
    Number(f32),
    Array,
    Nil,
    String(String),
}

impl BakhtScript {
    fn fcall(&mut self, argc: usize) {
        self.vm.fcall(argc)
    }
    fn reset(&mut self) {
        self.vm.reset();
    }
    fn load(&mut self, source: &str) -> CResult<()> {
        self.vm.reset();
        let text: Text = Arc::new(source.chars().collect());
        let scanner = Scanner::new(text.clone());
        let mut compiler = compiler::Compiler::new(text, scanner, BVM::default());
        compiler.compile()?;
        self.vm = compiler.vm();
        Ok(())
    }
    fn pop(&mut self) -> BakhtValue {
        match self.vm.pop() {
            vm::Value::String(s) => BakhtValue::String(s.to_string()),
            vm::Value::Array(_) => BakhtValue::Array,
            vm::Value::Nil => BakhtValue::Nil,
            vm::Value::Boolean(b) => BakhtValue::Boolean(b),
            vm::Value::Number(n) => BakhtValue::Number(n),
            vm::Value::Function(_) => BakhtValue::Function,
        }
    }
    fn push_nil(&mut self) {
        self.vm.push(vm::Value::Nil)
    }
    fn error(&self) -> Result<(), Error> {
        match self.vm.error() {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
    fn array_push(&mut self) {
        let ele = self.vm.pop();
        if let Value::Array(array) = self.vm.pop() {
            array.push(ele);
        }
    }
    fn array_pop(&mut self) {
        if let Value::Array(array) = self.vm.pop() {
            if let Some(ele) = array.pop() {
                self.vm.push(ele);
            }
        }
    }
    fn array_len(&mut self) {
        if let Value::Array(array) = self.vm.pop() {
            self.vm.push(Value::Number(array.len() as f32));
        }
    }
}

fn main() {
    let mut bs = BakhtScript::default();
    bs.load(
        std::fs::read_to_string("./local/source.bs")
            .unwrap()
            .as_str(),
    )
    .unwrap();
    bs.fcall(0);
    bs.error().unwrap();
    bs.reset();
}
