use crate::bin::Instruction;

pub enum Encoding {
    Fixed(u8),
    Variadic,
    None,
}

#[derive(Default)]
pub struct Bytecode {
    pub bytes: [u8; 9],
    pub len: u8,
}

pub fn encode_with_size(opcode: u8, operand: usize, size: u8) -> Bytecode {
    let x = match size {
        8 => 3,
        4 => 2,
        2 => 1,
        1 => 0,
        _ => panic!(),
    };
    let mut operand = operand;
    let mut bytecode = Bytecode::default();
    bytecode.len = size + 1;
    bytecode.bytes[0] = opcode | (x << 6);
    for i in 1..bytecode.len {
        bytecode.bytes[i as usize] = (operand & 0xff) as u8;
        operand = operand >> 8;
    }
    bytecode
}

pub fn encode(instruction: Instruction) -> Bytecode {
    let (opcode, operand, encoding) = instruction.encode_params();
    match encoding {
        Encoding::Fixed(size) => encode_with_size(opcode, operand, size),
        Encoding::Variadic => {
            if operand > 0xffffffff {
                encode_with_size(opcode, operand, 8)
            } else if operand > 0xffff {
                encode_with_size(opcode, operand, 4)
            } else if operand > 0xff {
                encode_with_size(opcode, operand, 2)
            } else {
                encode_with_size(opcode, operand, 1)
            }
        }
        Encoding::None => {
            let mut bytecode = Bytecode::default();
            bytecode.len = 1;
            bytecode.bytes[0] = opcode;
            bytecode
        }
    }
}
