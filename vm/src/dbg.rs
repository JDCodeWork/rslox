use crate::chunk::{Chunk, OpCode};

pub fn disasm_chunk(chunk: &Chunk, name: &'static str) {
    println!("== {} ==", name);

    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disasm_instr(offset, chunk);
    }
}

fn disasm_instr(offset: usize, chunk: &Chunk) -> usize {
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
    let value = chunk.constants[constant as usize];
    println!("{value}'");

    offset + 1
}
