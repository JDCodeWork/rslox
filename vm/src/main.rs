use vm::{
    chunk::{Chunk, OpCode},
    exec::VM,
};

fn main() {
    let mut c = Chunk::new();
    let mut temp_pointer;

    c.rles.add_rle(); // new line
    temp_pointer = c.add_const(2.0);
    c.write(OpCode::Constant);
    c.write(temp_pointer);

    temp_pointer = c.add_const(4.0);
    c.write(OpCode::Constant);
    c.write(temp_pointer);

    c.write(OpCode::Sub);

    c.write(OpCode::Return);
    VM::interpret(c);
}
