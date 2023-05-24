use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use crate::Error;

use super::BakhtScript;

pub(crate) type Native = fn(&mut BakhtScript);

#[derive(Clone, Copy)]
pub(crate) enum Function {
    Bakht { param_count: u8, address: usize },
    Native(Native),
}

#[derive(PartialEq)]
pub struct Array {
    inner: RefCell<Vec<Value>>,
}

impl Array {
    fn push(&self, value: Value) {
        self.inner.borrow_mut().push(value)
    }
    fn pop(&self) -> Option<Value> {
        self.inner.borrow_mut().pop()
    }
    fn len(&self) -> usize {
        self.inner.borrow().len()
    }
    fn get(&self, index: usize) -> Option<Value> {
        self.inner.borrow().get(index).cloned()
    }
    fn set(&self, index: usize, value: Value) -> bool {
        let arr = self.inner.borrow_mut();
        if arr.len() <= index {
            false
        } else {
            self.inner.borrow_mut()[index] = value;
            true
        }
    }
    fn new(array: Vec<Value>) -> Array {
        Array {
            inner: RefCell::new(array),
        }
    }
}
#[derive(PartialEq)]
pub struct Bstring {
    inner: RefCell<String>,
}

impl Bstring {
    fn push(&self, c: char) {
        self.inner.borrow_mut().push(c)
    }
    fn pop(&self) -> Option<char> {
        self.inner.borrow_mut().pop()
    }
    fn len(&self) -> usize {
        self.inner.borrow().len()
    }
    fn get(&self, index: usize) -> Option<Bstring> {
        self.inner
            .borrow()
            .chars()
            .nth(index)
            .map(|c| Bstring::new(c.to_string()))
    }
    fn set(&self, index: usize, c: char) -> bool {
        let arr = self.inner.borrow_mut();
        if arr.len() <= index {
            false
        } else {
            let c = c.to_string();
            self.inner
                .borrow_mut()
                .replace_range(index..index + 1, c.as_str());
            true
        }
    }
    fn value(&self) -> String {
        self.inner.borrow().to_string()
    }
    fn new(s: String) -> Bstring {
        Bstring {
            inner: RefCell::new(s),
        }
    }
}

#[derive(Clone)]
pub(crate) enum Value {
    String(Arc<Bstring>),
    Array(Arc<Array>),
    Nil,
    Boolean(bool),
    Number(f32),
    Function(Function),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Nil, Self::Nil) => true,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Function(l0), Self::Function(r0)) => match (l0, r0) {
                (
                    Function::Bakht {
                        param_count: _,
                        address: l0,
                    },
                    Function::Bakht {
                        param_count: _,
                        address: r0,
                    },
                ) => l0 == r0,
                (Function::Native(l0), Function::Native(r0)) => (*l0 as usize) == (*r0 as usize),
                _ => false,
            },
            _ => false,
        }
    }
}

pub(crate) trait VM {
    fn rodata_function(&mut self, param_count: u8, entry: bool) -> usize;
    fn emit(&mut self, bytecode: u8);
    fn rodata_number(&mut self, number: f32) -> usize;
    fn rodata_literal(&mut self, literal: String) -> usize;
    fn run(&mut self, argc: usize) {}
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
    error: Option<Error>,
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
        self.constants
            .push(Value::String(Arc::new(Bstring::new(literal))));
        idx
    }
    fn run(&mut self, argc: usize) {
        let entry = self.constants[self.entry].clone();
        self.push(entry);
        self.fcall(argc);
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
        Value::String(Arc::new(Bstring::new(s)))
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
        while self.error.is_none() && *self.ip() != 0 {
            let (opcode, operand) = self.fetch();
            match opcode {
                0 => self.i_add(),
                1 => self.i_sub(),
                2 => self.i_mult(),
                3 => self.i_div(),
                4 => self.i_eq(),
                5 => self.i_ne(),
                6 => self.i_ge(),
                7 => self.i_le(),
                8 => self.i_gt(),
                9 => self.i_lt(),
                18 => self.i_nil(),
                19 => self.i_true(),
                20 => self.i_false(),
                21 => self.i_anew(operand),
                22 => self.i_mod(),
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
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                let value = self.number(a + b);
                self.push(value)
            }
            _ => self.error = Some(Error::InvalidOperands),
        };
    }
    fn i_sub(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                let value = self.number(a - b);
                self.push(value)
            }
            _ => self.error = Some(Error::InvalidOperands),
        };
    }
    fn i_mult(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                let value = self.number(a * b);
                self.push(value)
            }
            _ => self.error = Some(Error::InvalidOperands),
        };
    }
    fn i_div(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                if b == 0.0 {
                    self.error = Some(Error::DivisionByZero);
                } else {
                    let value = self.number(a / b);
                    self.push(value)
                }
            }
            _ => self.error = Some(Error::InvalidOperands),
        };
    }
    fn i_mod(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                if b == 0.0 {
                    self.error = Some(Error::DivisionByZero);
                } else {
                    let value = self.number(a % b);
                    self.push(value)
                }
            }
            _ => self.error = Some(Error::InvalidOperands),
        };
    }
    fn i_true(&mut self) {
        self.push(Value::Boolean(true))
    }
    fn i_false(&mut self) {
        self.push(Value::Boolean(false))
    }
    fn i_nil(&mut self) {
        self.push(Value::Nil)
    }
    fn i_anew(&mut self, count: usize) {
        let mut elements = vec![];
        for _ in 0..count {
            elements.push(self.pop());
        }
        self.push(Value::Array(Arc::new(Array::new(elements))));
    }
    fn i_eq(&mut self) {
        let b = self.pop();
        let a = self.pop();
        self.push(Value::Boolean(a == b));
    }
    fn i_ne(&mut self) {
        let b = self.pop();
        let a = self.pop();
        self.push(Value::Boolean(a != b));
    }
    fn i_gt(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(l0), Value::Number(r0)) => self.push(Value::Boolean(l0 > r0)),
            (Value::String(l0), Value::String(r0)) => {
                self.push(Value::Boolean(l0.value() > r0.value()))
            }
            _ => self.error = Some(Error::InvalidOperands),
        }
    }
    fn i_lt(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(l0), Value::Number(r0)) => self.push(Value::Boolean(l0 > r0)),
            (Value::String(l0), Value::String(r0)) => {
                self.push(Value::Boolean(l0.value() > r0.value()))
            }
            _ => self.error = Some(Error::InvalidOperands),
        }
    }
    fn i_ge(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(l0), Value::Number(r0)) => self.push(Value::Boolean(l0 >= r0)),
            (Value::String(l0), Value::String(r0)) => {
                self.push(Value::Boolean(l0.value() >= r0.value()))
            }
            _ => self.error = Some(Error::InvalidOperands),
        }
    }
    fn i_le(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(l0), Value::Number(r0)) => self.push(Value::Boolean(l0 <= r0)),
            (Value::String(l0), Value::String(r0)) => {
                self.push(Value::Boolean(l0.value() <= r0.value()))
            }
            _ => self.error = Some(Error::InvalidOperands),
        }
    }
    fn i_pop(&mut self, count: usize) {
        for _ in 0..count {
            self.pop();
        }
    }
    fn i_get(&mut self) {
        let idx = self.pop();
        let val = self.pop();
        match (val, idx) {
            (Value::Array(v), Value::Number(i)) => match v.get(i as usize) {
                Some(ele) => self.push(ele),
                None => self.error = Some(Error::IndexOutOfBound),
            },
            (Value::String(v), Value::Number(i)) => match v.get(i as usize) {
                Some(ele) => self.push(Value::String(Arc::new(ele))),
                None => self.error = Some(Error::IndexOutOfBound),
            },
            _ => self.error = Some(Error::InvalidOperands),
        }
    }
    fn i_set(&mut self) {
        let ele = self.pop();
        let idx = self.pop();
        let val = self.pop();
        match (val, idx) {
            (Value::Array(v), Value::Number(i)) => {
                if v.set(i as usize, ele) {
                    self.push(Value::Array(v));
                } else {
                    self.error = Some(Error::IndexOutOfBound)
                }
            }
            _ => self.error = Some(Error::InvalidOperands),
        }
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
            error: None,
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
    Anew(usize) = 21,
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
            Instruction::Anew(o) => (21, Some(o)),
            Instruction::Mod => (22, None),
        }
    }
}
