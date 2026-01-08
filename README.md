# rslox

An implementation of the Lox language in Rust, following the book [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom.

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

## 🧾 Changelog

See `CHANGELOG.md` (e.g. v0.5.1: literals embedded into `TokenType`).

---

## 🛠️ Handy Commands

```bash
cargo run -q -- run --debug
cargo test
cargo build --release
```
