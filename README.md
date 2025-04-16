# rslox

This repository is an example of one of the ways in which the `lox` programming language could be implemented by following the steps described in the book [***"crafting interpreters"***](https://craftinginterpreters.com/) by *Robert Nystrom*

## How to Use

### ⚠️ **Development Alias Recommendation**

Since the project is currently in development, I recommend setting up an alias for the command `cargo run -q --` to make it easier to run the application while testing or trying it out. This will save time during development, as you won’t need to type the full command repeatedly.

To create the alias, add the following to your shell configuration file (e.g., `.bashrc`, `.zshrc`):

```bash
alias crq='cargo run -q --'
```

Once you’ve added the alias, you can use `crq` instead of `cargo run -q --` for the commands below.

### Show Help

To view the help message with available commands and options, use:

```bash
crq --help 
```

### Interactive Mode

To run the interpreter in interactive mode, use the following command:

```bash
crq run 
```

This will start the interpreter where you can type and execute code directly.

### Run From a File

To execute code from a file, use:

```bash
crq run -p <file_path>
```

Replace `<file_path>` with the path to your `.lox` file.

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

### 💻 Command Line Interface (CLI)
- `run` — Interactive prompt for parsing expressions and printing the AST
- `run -p <file_path>` — Parses and prints the AST for a file at the given path
- `tools gen-ast` — Generates internal AST data structures
- **Colored output** and enhanced CLI feedback
- 🚫 `debug` command planned for future versions

### 🖨️ Output
- Prints the **AST** representation of valid arithmetic expressions
- 🚫 No expression evaluation or runtime execution yet
