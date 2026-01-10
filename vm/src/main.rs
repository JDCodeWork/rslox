mod chunk;
mod dbg;
mod errors;

use chunk::{Byte, Chunk, OpCode};

use dbg::disasm_chunk;

fn main() {
    let mut c = Chunk::new();
    c.write(OpCode::Return as Byte);
    disasm_chunk(&c, "test");

    c.free();
}
