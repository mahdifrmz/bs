use super::BakhtScript;
use crate::Error;
use std::{cell::RefCell, sync::Arc};
pub(crate) type Native = fn(&mut BakhtScript);

const IADD: u8 = 0x0;
const ISUB: u8 = 0x1;
const IMULT: u8 = 0x2;
const IDIV: u8 = 0x3;
const IEQ: u8 = 0x4;
const INE: u8 = 0x5;
const IGE: u8 = 0x6;
const ILE: u8 = 0x7;
const IGT: u8 = 0x8;
const ILT: u8 = 0x9;
const ISET: u8 = 0xa;
const IGET: u8 = 0xb;
const IPOP: u8 = 0x2c;
const IRET: u8 = 0xd;
const ILOAD: u8 = 0x2e;
const ISTORE: u8 = 0x2f;
const ICALL: u8 = 0x30;
const IKONST: u8 = 0x31;
const INIL: u8 = 0x12;
const ITRUE: u8 = 0x13;
const IFALSE: u8 = 0x14;
const IANEW: u8 = 0x35;
const IMOD: u8 = 0x16;

#[derive(Clone, Copy)]
pub(crate) enum Function {
    Bakht { param_count: usize, address: usize },
    Native { param_count: usize, func: Native },
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

#[derive(Clone)]
pub(crate) enum Value {
    String(Arc<String>),
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
                (
                    Function::Native {
                        param_count: _,
                        func: l0,
                    },
                    Function::Native {
                        param_count: _,
                        func: r0,
                    },
                ) => (*l0 as usize) == (*r0 as usize),
                _ => false,
            },
            _ => false,
        }
    }
}

pub(crate) trait VM {
    fn rodata_function(&mut self, param_count: usize, entry: bool) -> usize;
    fn rodata_native(&mut self, native: Native, param_count: usize) -> usize;
    fn emit(&mut self, bytecode: u8);
    fn rodata_number(&mut self, number: f32) -> usize;
    fn rodata_literal(&mut self, literal: String) -> usize;
}

pub(crate) struct Frame {
    ip: usize,
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
        self.bin.push(bytecode);
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
    fn rodata_function(&mut self, param_count: usize, entry: bool) -> usize {
        let address = self.bin.len();
        let idx = self.constants.len();
        let val = Value::Function(Function::Bakht {
            param_count,
            address,
        });
        self.constants.push(val.clone());
        if entry {
            self.entry = idx;
        }
        self.push(val);
        idx
    }
    fn rodata_native(&mut self, func: Native, param_count: usize) -> usize {
        let idx = self.constants.len();
        self.constants
            .push(Value::Function(Function::Native { func, param_count }));
        idx
    }
}

impl BVM {
    pub fn fcall(&mut self, argc: usize) {
        self.i_call(argc)
    }
    pub fn reset(&mut self) {
        self.bin.clear();
        self.constants.clear();
        self.stack.clear();
        self.frames.clear();
        self.error = None;
        self.entry = 0;
        self.init();
    }
    pub fn init(&mut self) {
        self.frames.push(Frame { ip: 0, bp: 0 })
    }
    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }
    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
    fn ip(&mut self) -> &mut usize {
        &mut self.frames.last_mut().unwrap().ip
    }
    fn bp(&mut self) -> usize {
        self.frames.last_mut().unwrap().bp
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
                operand = operand << 8;
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
        while self.error.is_none() {
            let (opcode, operand) = self.fetch();
            match opcode {
                IADD => self.i_add(),
                ISUB => self.i_sub(),
                IMULT => self.i_mult(),
                IDIV => self.i_div(),
                IEQ => self.i_eq(),
                INE => self.i_ne(),
                IGE => self.i_ge(),
                ILE => self.i_le(),
                IGT => self.i_gt(),
                ILT => self.i_lt(),
                ISET => self.i_set(),
                IGET => self.i_get(),
                IPOP => self.i_pop(operand),
                IRET => self.i_ret(),
                ILOAD => self.i_load(operand),
                ISTORE => self.i_store(operand),
                ICALL => self.i_call(operand),
                IKONST => self.i_konst(operand),
                INIL => self.i_nil(),
                ITRUE => self.i_true(),
                IFALSE => self.i_false(),
                IANEW => self.i_anew(operand),
                IMOD => self.i_mod(),
                _ => panic!(),
            }
            if opcode == IRET {
                break;
            }
        }
    }
    fn i_load(&mut self, operand: usize) {
        let address = self.bp() + operand;
        let value = self.stack[address].clone();
        self.push(value)
    }
    fn i_store(&mut self, operand: usize) {
        let value = self.pop();
        let address = self.bp() + operand;
        self.stack[address] = value;
    }
    fn i_konst(&mut self, operand: usize) {
        let value = self.constants[operand].clone();
        self.push(value)
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
        elements.reverse();
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
            (Value::String(l0), Value::String(r0)) => self.push(Value::Boolean(l0 > r0)),
            _ => self.error = Some(Error::InvalidOperands),
        }
    }
    fn i_lt(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(l0), Value::Number(r0)) => self.push(Value::Boolean(l0 > r0)),
            (Value::String(l0), Value::String(r0)) => self.push(Value::Boolean(l0 > r0)),
            _ => self.error = Some(Error::InvalidOperands),
        }
    }
    fn i_ge(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(l0), Value::Number(r0)) => self.push(Value::Boolean(l0 >= r0)),
            (Value::String(l0), Value::String(r0)) => self.push(Value::Boolean(l0 >= r0)),
            _ => self.error = Some(Error::InvalidOperands),
        }
    }
    fn i_le(&mut self) {
        let b = self.pop();
        let a = self.pop();
        match (a, b) {
            (Value::Number(l0), Value::Number(r0)) => self.push(Value::Boolean(l0 <= r0)),
            (Value::String(l0), Value::String(r0)) => self.push(Value::Boolean(l0 <= r0)),
            _ => self.error = Some(Error::InvalidOperands),
        }
    }
    fn i_pop(&mut self, count: usize) {
        for _ in 0..count {
            self.pop();
        }
    }
    fn i_ret(&mut self) {
        let yld = self.pop();
        while self.sp() != self.bp() {
            self.pop();
        }
        self.frames.pop();
        self.push(yld);
    }
    fn i_get(&mut self) {
        let idx = self.pop();
        let val = self.pop();
        match (val, idx) {
            (Value::Array(v), Value::Number(i)) => match v.get(i as usize) {
                Some(ele) => self.push(ele),
                None => self.error = Some(Error::IndexOutOfBound),
            },
            (Value::String(v), Value::Number(i)) => match v.chars().nth(i as usize) {
                Some(ele) => self.push(Value::String(Arc::new(ele.to_string()))),
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
    fn sp(&self) -> usize {
        self.stack.len()
    }
    pub fn error(&self) -> Option<Error> {
        self.error.clone()
    }
    pub fn push_args(&mut self, argc: usize, param_count: usize) {
        if argc >= param_count as usize {
            for _ in 0..(argc - param_count as usize) {
                self.pop();
            }
        } else {
            for _ in 0..(param_count as usize - argc) {
                self.push(Value::Nil);
            }
        }
    }
    fn i_call(&mut self, argc: usize) {
        let func = self.stack.remove(self.sp() - 1 - argc);
        match func {
            Value::Function(f) => match f {
                Function::Bakht {
                    param_count,
                    address,
                } => {
                    self.push_args(argc, param_count);
                    self.frames.push(Frame {
                        ip: address,
                        bp: self.sp() - argc,
                    });
                    self.process();
                }
                Function::Native { param_count, func } => {
                    self.push_args(argc, param_count);
                    let vm = std::mem::take(self);
                    let mut bs = BakhtScript { vm };
                    func(&mut bs);
                    *self = bs.vm;
                }
            },
            _ => self.error = Some(Error::CallingNonFunction),
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
    Add = IADD,
    Sub = ISUB,
    Mult = IMULT,
    Div = IDIV,
    Eq = IEQ,
    Ne = INE,
    Ge = IGE,
    Le = ILE,
    Gt = IGT,
    Lt = ILT,
    Set = ISET,
    Get = IGET,
    Pop(usize) = IPOP,
    Ret = IRET,
    Load(usize) = ILOAD,
    Store(usize) = ISTORE,
    Call(usize) = ICALL,
    Konst(usize) = IKONST,
    Nil = INIL,
    True = ITRUE,
    False = IFALSE,
    Anew(usize) = IANEW,
    Mod = IMOD,
}

impl Instruction {
    pub(crate) fn encode_params(self) -> (u8, Option<usize>) {
        match self {
            Instruction::Add => (IADD, None),
            Instruction::Sub => (ISUB, None),
            Instruction::Mult => (IMULT, None),
            Instruction::Div => (IDIV, None),
            Instruction::Eq => (IEQ, None),
            Instruction::Ne => (INE, None),
            Instruction::Ge => (IGE, None),
            Instruction::Le => (ILE, None),
            Instruction::Gt => (IGT, None),
            Instruction::Lt => (ILT, None),
            Instruction::Set => (ISET, None),
            Instruction::Get => (IGET, None),
            Instruction::Pop(o) => (IPOP, Some(o)),
            Instruction::Ret => (IRET, None),
            Instruction::Load(o) => (ILOAD, Some(o)),
            Instruction::Store(o) => (ISTORE, Some(o)),
            Instruction::Call(o) => (ICALL, Some(o)),
            Instruction::Konst(o) => (IKONST, Some(o)),
            Instruction::Nil => (INIL, None),
            Instruction::True => (ITRUE, None),
            Instruction::False => (IFALSE, None),
            Instruction::Anew(o) => (IANEW, Some(o)),
            Instruction::Mod => (IMOD, None),
        }
    }
}
