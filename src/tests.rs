use super::{compiler::Compiler, vm::VM};
use crate::{bin::Instruction, scanner::Scanner};
use std::sync::Arc;

#[derive(Default)]
struct MockVM {
    bin: Vec<u8>,
    cidx: usize,
}

impl VM for MockVM {
    fn rodata_native(&mut self, _: crate::vm::Native, _: usize) -> usize {
        0
    }

    fn rodata_function(&mut self, _: usize, _: bool) -> usize {
        0
    }

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
    compiler.compile().unwrap();
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
