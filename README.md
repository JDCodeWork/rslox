# rslox

An implementation of the Lox language in Rust, following the book [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom.

**Version**: 1.0.0 - Full-featured interpreter with classes, inheritance, and complete language support.

---

## 🚀 Quick Start

```bash
git clone https://github.com/JDCodeWork/rslox.git
cd rslox
cargo build --release
./target/release/rslox run --help
```

## 🖥️ CLI Usage

Help:

```bash
cargo run -- --help
```

Start REPL:

```bash
cargo run -- run
```

Run a file:

```bash
cargo run -- run -p examples/expr.lox
```

## ✨ Features

### Language Support

- **Variables & Assignment**: Full variable declaration and dynamic typing
- **Control Flow**: if-else, while, for loops with break/continue
- **Functions**: First-class functions, closures, recursion
- **Classes**: Class declarations with constructors (`init`)
- **Inheritance**: Single inheritance with `super` keyword support
- **Methods**: Instance methods with `this` binding
- **Objects**: Dynamic property access and modification
- **Built-in Functions**: `clock()`, and standard library functions

### Architecture

- **Scanner**: Lexical analysis with comprehensive token support
- **Parser**: Recursive descent with full Lox grammar coverage
- **Resolver**: Static analysis pass for variable depth tracking
- **Interpreter**: Tree-walk execution with Arena-pattern environment management
- **Error Handling**: Detailed error reporting with line numbers and recovery

## 🧾 Changelog

See `CHANGELOG.md` for version history and `RELEASE_NOTES.md` for v1.0.0 details.

---

## 🛠️ Handy Commands

```bash
cargo run -q -- run --debug
cargo test
cargo build --release
```
