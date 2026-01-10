use crate::chunk::{Chunk, OpCode};

pub fn disasm_chunk(chunk: &Chunk, name: &'static str) {
    println!("== {} ==", name);

    for (offset, instr) in chunk.code.iter().enumerate() {
        disasm_instr(offset, *instr);
    }
}

fn disasm_instr(offset: usize, instr: u8) {
    print!("{:04} ", offset);

    let opcode = match OpCode::try_from(instr) {
        Ok(val) => val,
        Err(err) => return println!("{}", err),
    };

    match opcode {
        OpCode::Return => println!("Return"),
        OpCode::_COUNT => {}
    }
}
