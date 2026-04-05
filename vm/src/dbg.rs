use crate::chunk::{Chunk, OpCode};
use crate::scanner::Token;
use crate::values::{Object, Value};

pub fn disasm_chunk(chunk: &Chunk, name: &'static str) {
    println!("== {} ==", name);

    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disasm_instr(offset, chunk);
    }
}

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
        OpCode::Cons => const_instr("Constant", offset, chunk),
        OpCode::Pop => simple_instr("Pop", offset),

        OpCode::Print => simple_instr("Print", offset),
        OpCode::DefGlob => const_instr("DefGlob", offset, chunk),
        OpCode::GetGlob => const_instr("GetGlob", offset, chunk),

        OpCode::True => simple_instr("True", offset),
        OpCode::False => simple_instr("False", offset),
        OpCode::Nil => simple_instr("Nil", offset),
        OpCode::Eq => simple_instr("Eq", offset),
        OpCode::Greater => simple_instr("Greater", offset),
        OpCode::Less => simple_instr("Less", offset),

        OpCode::Neg => simple_instr("Negate", offset),
        OpCode::Not => simple_instr("Not", offset),
        OpCode::Add => simple_instr("Add", offset),
        OpCode::Sub => simple_instr("Sub", offset),
        OpCode::Mul => simple_instr("Mul", offset),
        OpCode::Div => simple_instr("Div", offset),
        // This should never happen
        OpCode::_COUNT => panic!(),
    }
}

fn simple_instr(name: &'static str, offset: usize) -> usize {
    println!("{name}");

    offset + 1
}

fn const_instr(name: &'static str, mut offset: usize, chunk: &Chunk) -> usize {
    print!("{name} ");
    offset += 1;

    let constant = chunk.code[offset];
    print!("({constant}) -> '");
    let value = chunk.constants[constant as usize].clone();
    println!("{value}'");

    offset + 1
}

pub fn dbg_mem(stack: &Vec<Value>, heap: &Vec<Object>) {
    for slot in stack.iter() {
        if let Value::Object(id) = slot {
            print!("[ {} ]", heap[id.0])
        } else {
            print!("[ {slot} ]");
        }
    }
    println!();
}

pub fn dbg_token(t: &Token, ln: &mut usize, source: &str) {
    if t.line != *ln {
        *ln = t.line;
        print!("{:4}  ", ln);
    } else {
        print!("   | ");
    }

    println!("{:2}  '{}'", t.kind.clone() as u8, t.lexeme(source))
}
