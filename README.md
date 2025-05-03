# rslox

This repository is an example implementation of the `lox` programming language, following the steps described in the book [***"Crafting Interpreters"***](https://craftinginterpreters.com/) by *Robert Nystrom*.

---

## 🚀 How to Use

### ⚠️ **Development Alias Recommendation**

Since the project is currently in development, it's convenient to create an alias for `cargo run -q --` to speed up testing.

Add the following to your shell configuration file (e.g., `.bashrc`, `.zshrc`):

```bash
alias crq='cargo run -q --'
```

Once added, you can use `crq` instead of `cargo run -q --` in the commands below.

---

### 📖 Show Help

To display the help message with available commands and options:

```bash
crq --help 
```

---

### 💻 Interactive Mode (REPL)

To start the interpreter in interactive mode:

```bash
crq run
```

This launches a REPL where you can type and execute code interactively.

---

### 📂 Run From a File

To execute code from a `.lox` file:

```bash
crq run -p <file_path>
```

Replace `<file_path>` with the path to your `.lox` file.

---

### 🐛 Debug Mode

To run in **debug mode**, which can optionally display tokens, the AST, or both:

```bash
crq run --debug
```

You can control what to display:

```bash
crq run --debug --show-ast
crq run --debug --show-tokens
crq run --debug --show-ast --show-tokens
```

✅ These options also work in interactive mode:

```bash
crq run --debug --show-ast
```

---

### 🛠️ Tools

Additional tools are available via the `tool` command:

```bash
crq tool <subcommand>
```

Available subcommands:

- `gen-ast` — Generates internal AST data structures

Example:

```bash
crq tool gen-ast
```

---

## 📋 Current Features

### 📝 Scanner
- **Single-character tokens:** `(` `)` `{` `}` `,` `.` `-` `+` `;` `/` `*`
- **One or two-character tokens:** `!` `!=` `=` `==` `>` `>=` `<` `<=`
- **Literals:**  
  - Identifiers  
  - Numbers  
  - Strings  
- **Keywords:**  
  `and`, `class`, `else`, `false`, `fun`, `for`, `if`, `nil`, `or`, `print`, `return`, `super`, `this`, `true`, `var`, `while`
- **Single-line comments:** `// ...`
- **Block comments:** `/* ... */`
- Emits an `EOF` token at the end of input
- Error handling for:
  - Invalid tokens

---

### 🌳 Parser
- Parses **arithmetic expressions** with `+`, `-`, `*`, `/`
- Respects **operator precedence and associativity**
- Supports **grouping** with parentheses `()`
- Generates an **Abstract Syntax Tree (AST)** for valid arithmetic expressions
- Error handling for:
  - Invalid tokens  
  - Syntax errors  
  - Unsupported keywords  
- 🚫 No parsing for logic operators (`and`, `or`) yet
- 🚫 No support for statements, declarations, or control structures yet

---

### 💻 Command Line Interface (CLI)
- `run` — Runs a `.lox` file or starts a REPL
  - `--debug` — Activates a debug mode with optional output controls
  - `--show-ast` — Shows the AST for each input
  - `--show-tokens` — Shows the list of tokens for each input
- `tool` — Executes development tools:
  - `gen-ast`
- **Colored output** and enhanced CLI feedback

---

### 🖨️ Output
- Prints the **AST** and/or **tokens** based on selected options
- 🚫 No expression evaluation or runtime execution yet