mod chunk;
mod dbg;
mod errors;
mod value;

use chunk::{Chunk, OpCode};

use dbg::disasm_chunk;

fn main() {
    let mut c = Chunk::new();

    let const_ = c.add_const(2.0);
    c.write(OpCode::Constant);
    c.write(const_);

    c.write(OpCode::Return);
    disasm_chunk(&c, "test");

    c.free();
}
