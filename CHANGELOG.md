# Changelog

All notable changes to this project will be documented in this file.  
This project adheres to [Semantic Versioning](https://semver.org).  
The format is inspired by [Keep a Changelog](https://keepachangelog.com).

---

## [Unreleased]

### Added

- Parser implementation with expression support
- Interpreter with evaluation system
- Simple panic mode for errors
- Runtime error handling with specific error types

### Changed

- Refactored `Literal` from struct to enum with typed variants
- Refactored error system with separated error types:
  - `ScanError` for lexical analysis errors
  - `ParseError` for syntax errors
  - `RuntimeError` for execution errors
  - `SystemError` for file and IO errors
  - `CLIError` for CLI-related errors

## [0.5.1] - 2025-09-13

### Changed

- Refactor CLI handling system to improve functionality and structure.
- Literals embedded into `TokenType`

## [0.5.0] - 2025-01-25

### Added

- Enhance CLI: Added parameters, commands, and color support
- Tool for generate AST
- Tool for view Ast

## [0.4.1] - 2025-01-16

### Fixed

- Special characters

## [0.4.0] - 2025-01-15

### Added

- Scanner support:
  - Reserved words
  - identifiers
  - block comments

## [0.3.0] - 2025-01-13

### Added

- Scanner support: literal strings and literal numbers

### Changed

- Reading files only supports extensions ending in `.lox`

## [0.2.0] - 2025-01-11

### Added

- Basic scanner and token system
- Basic error system

## [0.1.0] - 2025-01-06

### Added

- Support source code from command line and give it a path to a file
