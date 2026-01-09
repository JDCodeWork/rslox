# rslox - Tree Walk Interpreter

**Version**: 1.0.0 - Full-featured interpreter with classes, inheritance, and complete language support.

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

---

## 🖥️ CLI Usage

Help:

```bash
./tw --help
```

Debug:

```bash
./tw --debug
```

Start REPL:

```bash
./tw run
```

Run a file:

```bash
./tw run --path ./playground/factorial.lox
```

## 🧾 Changelog

See `CHANGELOG.md` for version history.
