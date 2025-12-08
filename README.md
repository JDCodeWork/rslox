# rslox

An implementation of the Lox language in Rust, following the book [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom.

---

## 🎯 Goals

1. Practice interpreter construction.
2. Apply idiomatic Rust patterns.
3. Incrementally evolve into a full interpreter.

---

## ⚙️ Requirements

- Rust 1.70+ (edition 2021) — install via <https://rustup.rs>

---

## 🚀 Quick Start

```bash
git clone https://github.com/JDCodeWork/rslox.git
cd rslox
cargo build --release
./target/release/rslox run --help
```

### Optional dev alias

```bash
echo "alias crq='cargo run -q --'" >> ~/.zshrc  # or ~/.bashrc
exec $SHELL -l
```

---

## 🖥️ CLI Usage

Help:

```bash
crq --help
```

Start REPL:

```bash
crq run
```

Run a file:

```bash
crq run -p examples/expr.lox
```

If you omit `.lox`, it will be appended automatically.

### Debug mode

```bash
crq run --debug --show-tokens
crq run --debug --show-ast
crq run --debug --show-ast --show-tokens
```

Works inside the REPL as well.

### Tools (subcommand `tool`)

Generate AST boilerplate (requires an output dir):

```bash
crq tool gen-ast ./out
```

---

## 📦 Project Structure

```text
src/
  cli.rs       # CLI (clap)
  errors.rs    # Error types
  lox/
    scanner.rs # Source -> tokens
    token.rs   # Token & TokenType
    parser.rs  # Recursive descent expression parser
    expr.rs    # AST node definitions & pretty printing
  tools/
    mod.rs     # AstPrinter + AST code generator
```

---

## 🔤 Tokens Supported

- Single-char: `(` `)` `{` `}` `,` `.` `-` `+` `;` `/` `*`
- One/two-char: `!` `!=` `=` `==` `>` `>=` `<` `<=`
- Literals: identifiers, numbers (`f64`), strings
- Keywords: `and class else false fun for if nil or print return super this true var while`
- Comments: `//` (line) and `/* ... */` (block)
- EOF sentinel

Literals are stored directly in `TokenType` as `String(String)` and `Number(f64)` (no separate literal field).

---

## 🌳 Interpreter Status

Features:

- Arithmetic expressions: `+ - * /`
- Correct precedence & associativity
- Grouping via `(...)`
- Literals: numbers, strings, `true`, `false`, `nil`
- AST generation (printed in Lisp-like form)
- **Evaluation**: Tree-walk interpreter
- **Control Flow**: `if-else`, `while`, `for`
- **Logical Operators**: `and`, `or`
- **Statements**: `print`, blocks, variable declarations

(temporal) Limitations:

- No functions or classes

---

## 🧪 Examples

Input:

```text
1 + 2 * (3 - 4)
```

With `--debug --show-ast --show-tokens`:

```text
 INFO  Token( type: Number(1.0), literal: (1), lexeme: 1 ) at line 1
 INFO  Token( type: Plus, literal: (), lexeme: + ) at line 1
 INFO  Token( type: Number(2.0), literal: (2), lexeme: 2 ) at line 1
 INFO  Token( type: Star, literal: (), lexeme: * ) at line 1
 INFO  Token( type: LeftParen, literal: (), lexeme: ( ) at line 1
 INFO  Token( type: Number(3.0), literal: (3), lexeme: 3 ) at line 1
 INFO  Token( type: Minus, literal: (), lexeme: - ) at line 1
 INFO  Token( type: Number(4.0), literal: (4), lexeme: 4 ) at line 1
 INFO  Token( type: RightParen, literal: (), lexeme: ) ) at line 1
 INFO  Token( type: EOF, literal: (), lexeme:  ) at line 2

 INFO  AST -> (+ 1 (* 2 (group (- 3 4))))
```

Another:

```text
-("hello" + " world")
```

AST:

```text
 INFO  Token( type: Minus, literal: (), lexeme: - ) at line 1
 INFO  Token( type: LeftParen, literal: (), lexeme: ( ) at line 1
 INFO  Token( type: String("hello"), literal: (hello), lexeme: "hello" ) at line 1
 INFO  Token( type: Plus, literal: (), lexeme: + ) at line 1
 INFO  Token( type: String(" world"), literal: ( world), lexeme: " world" ) at line 1
 INFO  Token( type: RightParen, literal: (), lexeme: ) ) at line 1
 INFO  Token( type: EOF, literal: (), lexeme:  ) at line 2

 INFO  AST -> (- (group (+ hello  world)))
```

---

## 🛠️ AST Generator

Generates enums/structs + `new` constructors from concise type descriptions to reduce boilerplate while iterating on the language.

Conceptual description example:

```text
Expr -> Binary left: Expr, operator: Token, right: Expr
```

---

## 🚨 Error Handling

Reports:

- Unexpected character
- Unterminated string
- Unknown token / type during parse
- Syntax errors (missing `)` etc.)

Colored output via `owo-colors`.

---

## 💡 Technical Notes

- `TokenType` embeds literal values (removed dynamic literal field).
- AST printing uses a parenthesized style: `(+ 1 (* 2 3))`.
- `AstPrinter` centralizes string representation.

## 🧾 Changelog

See `CHANGELOG.md` (e.g. v0.5.1: literals embedded into `TokenType`).

---

## 🛠️ Handy Commands

```bash
cargo run -q -- run --debug --show-tokens --show-ast
cargo test
cargo build --release
```
