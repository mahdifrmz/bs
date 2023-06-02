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
        }
    }
}

impl Instruction {
    pub fn encode_params(&self) -> (u8, Option<usize>) {
        match self {
			Instruction::Nop => (INOP, None),
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
			Instruction::Pop(operand) => (IPOP, Some(*operand)),
			Instruction::Ret => (IRET, None),
			Instruction::Load(operand) => (ILOAD, Some(*operand)),
			Instruction::Store(operand) => (ISTORE, Some(*operand)),
			Instruction::Call(operand) => (ICALL, Some(*operand)),
			Instruction::Konst(operand) => (IKONST, Some(*operand)),
			Instruction::Nil => (INIL, None),
			Instruction::True => (ITRUE, None),
			Instruction::False => (IFALSE, None),
			Instruction::Anew(operand) => (IANEW, Some(*operand)),
			Instruction::Mod => (IMOD, None),
        }
    }
}
