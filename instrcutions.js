const OPTYPE_USIZE = 'usize'
const OPTYPE_U16 = 'u16'

const instructions = [
    ['nop'],
    ['add'],
    ['sub'],
    ['mult'],
    ['div'],
    ['eq'],
    ['ne'],
    ['ge'],
    ['le'],
    ['gt'],
    ['lt'],
    ['set'],
    ['get'],
    ['pop',OPTYPE_USIZE],
    ['ret'],
    ['load',OPTYPE_USIZE],
    ['store',OPTYPE_USIZE],
    ['call',OPTYPE_USIZE],
    ['konst',OPTYPE_USIZE],
    ['nil'],
    ['true'],
    ['false'],
    ['anew',OPTYPE_USIZE],
    ['mod'],
    ['jmp',OPTYPE_U16],
    ['cjmp',OPTYPE_U16],
]

function generate_to_string()
{
    let cases = ''
    for (let i = 0;i<instructions.length;i++)
    {
        const [name,operand] = instructions[i]
        cases += `\t\t\tInstruction::${name[0].toUpperCase()}${name.slice(1)}`
        if (operand) {
            cases += `(operand)`
        }
        cases += ` => format!("${name}`
        if (operand) {
            cases += '({})'
        }
        cases += '"'
        if (operand) {
            cases += `, operand`
        }
        cases += '),\n'
    }
    cases = cases.slice(0,cases.length-1)
    return `
impl ToString for Instruction {
    fn to_string(&self) -> String {
        match self {
${cases}
        }
    }
}`
}

function generate_encode_params()
{
    let cases = ''
    for (let i = 0;i<instructions.length;i++)
    {
        const [name,operand] = instructions[i]
        cases += `\t\t\tInstruction::${name[0].toUpperCase()}${name.slice(1)}`
        if (operand) {
            cases += `(operand)`
        }
        cases += ` => (I${name.toUpperCase()}`
        if (operand) {
            cases += `, *operand as usize`
        }
        else {
            cases += ', 0usize'
        }
        cases += `, ${operand ? (operand == OPTYPE_U16 ? 'Encoding::Fixed(2)' : 'Encoding::Variadic') : 'Encoding::None'}),\n`
    }
    cases = cases.slice(0,cases.length-1)
    return `
impl Instruction {
    pub fn encode_params(&self) -> (u8, usize, Encoding) {
        match self {
${cases}
        }
    }
}`
}

function generate_enum()
{
    let cases = ''
    for (let i = 0;i<instructions.length;i++)
    {
        const [name,operand] = instructions[i]
        cases += `    ${name[0].toLocaleUpperCase()}${name.slice(1)}`
        if (operand) {
            cases += `(${operand})`
        }
        cases += ` = I${name.toUpperCase()},\n`
    }
    cases = cases.slice(0,cases.length-1)
    return `#[repr(u8)]
pub enum Instruction {
${cases}
}`
}

function generate_constants()
{
    let cases = ''
    for (let i = 0;i<instructions.length;i++)
    {
        const [name,operand] = instructions[i]
        cases += `pub const I${name.toUpperCase()}: u8 = ${i | (operand ? 0b00100000 : 0)};\n`
    }
    return cases
}

function generate_imports()
{
    return 'use crate::assemble::Encoding;\n'
}

console.log(generate_imports())
console.log(generate_constants())
console.log(generate_enum())
console.log(generate_to_string())
console.log(generate_encode_params())