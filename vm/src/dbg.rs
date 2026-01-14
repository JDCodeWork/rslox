#[allow(unused_imports)]
use crate::chunk::{Chunk, OpCode, Value};

#[cfg(not(feature = "dbg"))]
#[allow(unused_variables, unused_mut)]
#[inline]
pub fn disasm_chunk(chunk: &Chunk, name: &'static str) {}

#[cfg(feature = "dbg")]
pub fn disasm_chunk(chunk: &Chunk, name: &'static str) {
    println!("== {} ==", name);

    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disasm_instr(offset, chunk);
    }
}

#[cfg(not(feature = "dbg"))]
#[allow(unused_variables, unused_mut)]
#[inline]
pub fn disasm_instr(offset: usize, chunk: &Chunk) -> usize {
    0
}

#[cfg(feature = "dbg")]
pub fn disasm_instr(offset: usize, chunk: &Chunk) -> usize {
    print!("{:04} ", offset);

    let opcode = match OpCode::try_from(chunk.code[offset]) {
        Err(err) => {
            println!("{err}");
            return offset + 1;
        }
        Ok(val) => val,
    };

    let runs = &chunk.rles;
    if offset > 0 && runs.get_ln(offset) == runs.get_ln(offset - 1) {
        print!("   | ");
    } else {
        print!("{:4} ", runs.get_ln(offset));
    }

    match opcode {
        OpCode::Return => simple_instr("Return", offset),
        OpCode::Constant => const_instr("Constant", offset, chunk),
        OpCode::Negate => simple_instr("Negate", offset),
        OpCode::Add => simple_instr("Add", offset),
        OpCode::Sub => simple_instr("Sub", offset),
        OpCode::Mul => simple_instr("Mul", offset),
        OpCode::Div => simple_instr("Div", offset),
        // This should never happen
        OpCode::_COUNT => panic!(),
    }
}

#[cfg(not(feature = "dbg"))]
#[allow(dead_code, unused_variables, unused_mut)]
#[inline]
fn simple_instr(name: &'static str, offset: usize) -> usize {
    0
}

#[cfg(feature = "dbg")]
fn simple_instr(name: &'static str, offset: usize) -> usize {
    println!("{name}");

    offset + 1
}

#[cfg(not(feature = "dbg"))]
#[allow(dead_code, unused_variables, unused_mut)]
#[inline]
fn const_instr(name: &'static str, mut offset: usize, chunk: &Chunk) -> usize {
    0
}

#[cfg(feature = "dbg")]
fn const_instr(name: &'static str, mut offset: usize, chunk: &Chunk) -> usize {
    print!("{name} ");
    offset += 1;

    let constant = chunk.code[offset];
    print!("({constant}) -> '");
    let value = chunk.constants[constant as usize];
    println!("{value}'");

    offset + 1
}
#[cfg(not(feature = "dbg"))]
#[allow(dead_code, unused_variables, unused_mut)]
#[inline]
pub fn dbg_stack(stack: &Vec<Value>) {}

#[cfg(feature = "dbg")]
pub fn dbg_stack(stack: &Vec<Value>) {
    for slot in stack.iter() {
        print!("[ {slot} ]");
    }
    println!();
}
