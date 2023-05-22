use super::BakhtScript;

pub(crate) type Native = fn(&mut BakhtScript);

pub(crate) enum Function {
    Bakht { param_count: u8, address: usize },
    Native(Native),
}

pub(crate) enum Value {
    String(String),
    Array(Vec<Value>),
    Nil,
    Boolean(bool),
    Number(f32),
    Function(Function),
}

pub(crate) trait VM {
    fn function(&mut self, param_count: u8);
    fn emit(&mut self, bytecode: u8);
    fn rodata_number(&mut self, number: f32) -> usize;
    fn rodata_literal(&mut self, literal: String) -> usize;
    fn run(&mut self) {}
}

#[derive(Default)]
pub(crate) struct BVM {}

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
pub(crate) enum Instruction {
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
    Pop(usize) = 12,
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
    pub(crate) fn encode_params(self) -> (u8, Option<usize>) {
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
            Instruction::Pop(o) => (12, Some(o)),
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
