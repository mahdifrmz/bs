use std::sync::Arc;

use super::BakhtScript;

pub(crate) type Native = fn(&mut BakhtScript);

#[derive(Clone, Copy)]
pub(crate) enum Function {
    Bakht { param_count: u8, address: usize },
    Native(Native),
}

#[derive(Clone)]
pub(crate) enum Value {
    String(Arc<String>),
    Array(Arc<Vec<Value>>),
    Nil,
    Boolean(bool),
    Number(f32),
    Function(Function),
}

pub(crate) trait VM {
    fn rodata_function(&mut self, param_count: u8, entry: bool) -> usize;
    fn emit(&mut self, bytecode: u8);
    fn rodata_number(&mut self, number: f32) -> usize;
    fn rodata_literal(&mut self, literal: String) -> usize;
    fn run(&mut self, args: Vec<String>) {}
    fn reset(&mut self) {}
}

pub(crate) struct Frame {
    ip: usize,
    sp: usize,
    bp: usize,
}

pub(crate) struct BVM {
    stack: Vec<Value>,
    bin: Vec<u8>,
    constants: Vec<Value>,
    frames: Vec<Frame>,
    entry: usize,
}

impl VM for BVM {
    // FIXME
    fn emit(&mut self, bytecode: u8) {
        print!("{}:", bytecode)
    }
    fn rodata_number(&mut self, number: f32) -> usize {
        let idx = self.constants.len();
        self.constants.push(Value::Number(number));
        idx
    }
    fn rodata_literal(&mut self, literal: String) -> usize {
        let idx = self.constants.len();
        self.constants.push(Value::String(Arc::new(literal)));
        idx
    }
    fn run(&mut self, args: Vec<String>) {
        let entry = self.constants[self.entry].clone();
        self.push(entry);
        let mut args = args;
        for arg in args.drain(..) {
            let arg = self.string(arg);
            self.push(arg);
        }
        self.fcall(args.len());
        self.process();
    }
    fn rodata_function(&mut self, param_count: u8, entry: bool) -> usize {
        let address = self.bin.len();
        let idx = self.constants.len();
        self.constants.push(Value::Function(Function::Bakht {
            param_count,
            address,
        }));
        if entry {
            self.entry = idx;
        }
        idx
    }
    fn reset(&mut self) {
        self.bin.clear();
        self.constants.clear();
        self.stack.clear();
        self.init();
    }
}

impl BVM {
    fn init(&mut self) {
        self.frames.push(Frame {
            ip: 0,
            sp: 0,
            bp: 0,
        })
    }
    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
    fn fcall(&mut self, arg_count: usize) {
        // TODO
    }
    fn string(&self, s: String) -> Value {
        Value::String(Arc::new(s))
    }
    fn ip(&mut self) -> &mut usize {
        &mut self.frames.last_mut().unwrap().ip
    }
    fn read(&mut self) -> u8 {
        let ip = *self.ip();
        let opcode = self.bin[ip];
        *self.ip() += 1;
        opcode
    }
    fn fetch(&mut self) -> (u8, usize) {
        let mut opcode = self.read();
        let operand = if opcode & 0b_0010_0000 > 0 {
            let operand_count = (opcode & 0b_1100_0000) >> 6;
            let operand_count = 1 << operand_count;
            let mut operand = 0usize;
            for _ in 0..operand_count {
                operand = operand << 1;
                operand += self.read() as usize;
            }
            opcode &= 0b_0011_11111;
            operand
        } else {
            0
        };
        (opcode, operand)
    }
    fn process(&mut self) {
        while *self.ip() != 0 {
            let (opcode, operand) = self.fetch();
            match opcode {
                0 => self.i_add(),
                _ => panic!(), // TODO
            }
        }
    }
    fn number(&mut self, value: f32) -> Value {
        Value::Number(value)
    }
    fn i_add(&mut self) {
        let b = self.pop();
        let a = self.pop();
        let value = match (a, b) {
            (Value::Number(a), Value::Number(b)) => self.number(a + b),
            _ => Value::Nil, // TODO
        };
        self.push(value)
    }
}

impl Default for BVM {
    fn default() -> Self {
        let mut bvm = Self {
            stack: Default::default(),
            bin: Default::default(),
            constants: Default::default(),
            frames: Default::default(),
            entry: Default::default(),
        };
        bvm.init();
        bvm
    }
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
        }
    }
}
