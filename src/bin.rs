use crate::assemble::Encoding;

pub const INOP: u8 = 0;
pub const IADD: u8 = 1;
pub const ISUB: u8 = 2;
pub const IMULT: u8 = 3;
pub const IDIV: u8 = 4;
pub const IEQ: u8 = 5;
pub const INE: u8 = 6;
pub const IGE: u8 = 7;
pub const ILE: u8 = 8;
pub const IGT: u8 = 9;
pub const ILT: u8 = 10;
pub const ISET: u8 = 11;
pub const IGET: u8 = 12;
pub const IPOP: u8 = 45;
pub const IRET: u8 = 14;
pub const ILOAD: u8 = 47;
pub const ISTORE: u8 = 48;
pub const ICALL: u8 = 49;
pub const IKONST: u8 = 50;
pub const INIL: u8 = 19;
pub const ITRUE: u8 = 20;
pub const IFALSE: u8 = 21;
pub const IANEW: u8 = 54;
pub const IMOD: u8 = 23;
pub const IJMP: u8 = 56;
pub const ICJMP: u8 = 57;

#[repr(u8)]
pub enum Instruction {
    Nop = INOP,
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
    Jmp(u16) = IJMP,
    Cjmp(u16) = ICJMP,
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        match self {
            Instruction::Nop => format!("nop"),
            Instruction::Add => format!("add"),
            Instruction::Sub => format!("sub"),
            Instruction::Mult => format!("mult"),
            Instruction::Div => format!("div"),
            Instruction::Eq => format!("eq"),
            Instruction::Ne => format!("ne"),
            Instruction::Ge => format!("ge"),
            Instruction::Le => format!("le"),
            Instruction::Gt => format!("gt"),
            Instruction::Lt => format!("lt"),
            Instruction::Set => format!("set"),
            Instruction::Get => format!("get"),
            Instruction::Pop(operand) => format!("pop({})", operand),
            Instruction::Ret => format!("ret"),
            Instruction::Load(operand) => format!("load({})", operand),
            Instruction::Store(operand) => format!("store({})", operand),
            Instruction::Call(operand) => format!("call({})", operand),
            Instruction::Konst(operand) => format!("konst({})", operand),
            Instruction::Nil => format!("nil"),
            Instruction::True => format!("true"),
            Instruction::False => format!("false"),
            Instruction::Anew(operand) => format!("anew({})", operand),
            Instruction::Mod => format!("mod"),
            Instruction::Jmp(operand) => format!("jmp({})", operand),
            Instruction::Cjmp(operand) => format!("cjmp({})", operand),
        }
    }
}

impl Instruction {
    pub fn encode_params(&self) -> (u8, usize, Encoding) {
        match self {
            Instruction::Nop => (INOP, 0usize, Encoding::None),
            Instruction::Add => (IADD, 0usize, Encoding::None),
            Instruction::Sub => (ISUB, 0usize, Encoding::None),
            Instruction::Mult => (IMULT, 0usize, Encoding::None),
            Instruction::Div => (IDIV, 0usize, Encoding::None),
            Instruction::Eq => (IEQ, 0usize, Encoding::None),
            Instruction::Ne => (INE, 0usize, Encoding::None),
            Instruction::Ge => (IGE, 0usize, Encoding::None),
            Instruction::Le => (ILE, 0usize, Encoding::None),
            Instruction::Gt => (IGT, 0usize, Encoding::None),
            Instruction::Lt => (ILT, 0usize, Encoding::None),
            Instruction::Set => (ISET, 0usize, Encoding::None),
            Instruction::Get => (IGET, 0usize, Encoding::None),
            Instruction::Pop(operand) => (IPOP, *operand as usize, Encoding::Variadic),
            Instruction::Ret => (IRET, 0usize, Encoding::None),
            Instruction::Load(operand) => (ILOAD, *operand as usize, Encoding::Variadic),
            Instruction::Store(operand) => (ISTORE, *operand as usize, Encoding::Variadic),
            Instruction::Call(operand) => (ICALL, *operand as usize, Encoding::Variadic),
            Instruction::Konst(operand) => (IKONST, *operand as usize, Encoding::Variadic),
            Instruction::Nil => (INIL, 0usize, Encoding::None),
            Instruction::True => (ITRUE, 0usize, Encoding::None),
            Instruction::False => (IFALSE, 0usize, Encoding::None),
            Instruction::Anew(operand) => (IANEW, *operand as usize, Encoding::Variadic),
            Instruction::Mod => (IMOD, 0usize, Encoding::None),
            Instruction::Jmp(operand) => (IJMP, *operand as usize, Encoding::Fixed(2)),
            Instruction::Cjmp(operand) => (ICJMP, *operand as usize, Encoding::Fixed(2)),
        }
    }
}
