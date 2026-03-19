# Chapter 9: The vos script Language

## Learning Objectives

After completing this chapter, you will understand:
- Programming language design principles
- Lexical analysis (tokenization)
- Abstract Syntax Trees (ASTs)
- Language syntax and grammar
- How compilers and interpreters work

## Introduction

Every operating system needs a way for users to write programs and scripts. While assembly is powerful, it's tedious for everyday tasks. High-level languages make programming accessible and productive.

In this chapter, we'll explore **vos script**, VOS's scripting language. It's inspired by TypeScript and Ruby, featuring:
- Clean, readable syntax
- Static typing with inference
- First-class functions
- Modern control flow

We'll build the **lexer** (tokenizer) and design the **AST** (Abstract Syntax Tree) that represents programs internally.

## Language Design

### Goals

**vos script** aims to be:
1. **Simple**: Easy to learn and read
2. **Safe**: Catch errors at compile time
3. **Expressive**: Write less code to do more
4. **Familiar**: Syntax similar to popular languages

### Example Program

```vos
// Calculate factorial
fn factorial(n: int) -> int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

fn main() -> int {
    let result = factorial(5)
    print("Factorial of 5 is")
    print(result)
    return 0
}
```

Notice:
- Type annotations (`: int`, `-> int`)
- No semicolons (newline-terminated)
- Familiar keywords (`fn`, `let`, `if`, `return`)
- Clean, readable structure

## Language Features

### Types

```vos
// Primitive types
let x: int = 42           // 32-bit signed integer
let pi: float = 3.14159   // Floating point
let flag: bool = true     // Boolean
let ch: char = 'A'        // Single character
let msg: string = "Hello" // String

// Type inference
let y = 10        // Inferred as int
let name = "VOS"  // Inferred as string
```

### Variables

```vos
// Mutable variables with let
let count = 0
count = count + 1

// Immutable constants with const
const MAX_SIZE = 1000
// MAX_SIZE = 2000  // Error!
```

### Functions

```vos
// Function with parameters and return type
fn add(a: int, b: int) -> int {
    return a + b
}

// Function without return value
fn greet(name: string) {
    print("Hello, ")
    print(name)
}

// Main function (entry point)
fn main() -> int {
    let sum = add(5, 3)
    greet("VOS")
    return 0
}
```

### Control Flow

```vos
// If-else
if x > 10 {
    print("Large")
} else if x > 5 {
    print("Medium")
} else {
    print("Small")
}

// While loop
let i = 0
while i < 10 {
    print(i)
    i = i + 1
}

// For loop (planned)
for num in [1, 2, 3, 4, 5] {
    print(num)
}
```

### Expressions

```vos
// Arithmetic
let sum = a + b
let product = x * y
let remainder = n % 10

// Comparison
let is_greater = x > y
let is_equal = a == b

// Logical
let both = x > 0 && y > 0
let either = flag1 || flag2
```

## Compiler Pipeline

Building a language involves several stages:

```
Source Code (.vos)
        │
        ▼
┌────────────────┐
│     Lexer      │  Tokenize into tokens
└────────────────┘
        │
        ▼
┌────────────────┐
│     Parser     │  Build Abstract Syntax Tree
└────────────────┘
        │
        ▼
┌────────────────┐
│ Type Checker   │  Verify types are correct
└────────────────┘
        │
        ▼
┌────────────────┐
│    Compiler    │  Generate machine code
│       or       │     or
│  Interpreter   │  Execute directly
└────────────────┘
        │
        ▼
    Execution
```

We'll focus on the **lexer** and **AST design** in this chapter.

## Lexical Analysis (Tokenization)

The **lexer** (or tokenizer) converts source code text into a sequence of tokens.

### What is a Token?

A **token** is the smallest meaningful unit in a language:

```vos
let x = 42
```

Tokens:
1. `let` - Keyword
2. `x` - Identifier
3. `=` - Operator
4. `42` - Integer literal

### Token Structure

```rust
pub struct Token {
    pub kind: TokenKind,       // What kind of token
    pub lexeme: String,        // The actual text
    pub line: usize,           // Line number
    pub column: usize,         // Column number
}
```

Example:
```rust
Token {
    kind: TokenKind::Integer(42),
    lexeme: "42".to_string(),
    line: 1,
    column: 9,
}
```

### Token Kinds

```rust
pub enum TokenKind {
    // Literals
    Integer(i32),
    Float(String),
    String(String),
    True,
    False,

    // Keywords
    Let,
    Const,
    Fn,
    If,
    Else,
    While,
    For,
    Return,

    // Types
    TypeInt,
    TypeFloat,
    TypeBool,
    TypeString,

    // Operators
    Plus, Minus, Star, Slash,
    Equal, EqualEqual, BangEqual,
    Less, LessEqual, Greater, GreaterEqual,
    And, Or, Bang,

    // Delimiters
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Comma, Colon, Semicolon,

    // Special
    Identifier(String),
    Newline,
    Eof,
}
```

## Implementing the Lexer

### Lexer State

```rust
pub struct Lexer {
    source: Vec<char>,    // Source code as characters
    current: usize,       // Current position
    line: usize,          // Current line
    column: usize,        // Current column
}
```

### Main Tokenization Loop

```rust
pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();

    loop {
        let token = self.next_token()?;
        let is_eof = token.kind == TokenKind::Eof;
        tokens.push(token);

        if is_eof {
            break;
        }
    }

    Ok(tokens)
}
```

### Processing One Token

```rust
fn next_token(&mut self) -> Result<Token, String> {
    // Skip whitespace
    self.skip_whitespace();

    // Check for EOF
    if self.is_at_end() {
        return Ok(self.make_token(TokenKind::Eof, ""));
    }

    let c = self.advance();

    // Numbers
    if c.is_ascii_digit() {
        return self.number(c);
    }

    // Identifiers and keywords
    if c.is_alphabetic() || c == '_' {
        return self.identifier(c);
    }

    // Strings
    if c == '"' {
        return self.string();
    }

    // Operators...
    // Delimiters...
}
```

### Tokenizing Numbers

```rust
fn number(&mut self, first: char) -> Result<Token, String> {
    let mut num_str = first.to_string();

    // Collect digits
    while let Some(c) = self.peek() {
        if c.is_ascii_digit() {
            num_str.push(c);
            self.advance();
        } else {
            break;
        }
    }

    // Check for decimal point
    if self.peek() == Some('.') &&
       self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
        // Float
        num_str.push('.');
        self.advance();

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::new(TokenKind::Float(num_str.clone()), num_str, ...))
    } else {
        // Integer
        let value = num_str.parse::<i32>()?;
        Ok(Token::new(TokenKind::Integer(value), num_str, ...))
    }
}
```

### Tokenizing Identifiers

```rust
fn identifier(&mut self, first: char) -> Result<Token, String> {
    let mut ident = first.to_string();

    while let Some(c) = self.peek() {
        if c.is_alphanumeric() || c == '_' {
            ident.push(c);
            self.advance();
        } else {
            break;
        }
    }

    // Check if it's a keyword
    let kind = match ident.as_str() {
        "let" => TokenKind::Let,
        "const" => TokenKind::Const,
        "fn" => TokenKind::Fn,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "return" => TokenKind::Return,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "int" => TokenKind::TypeInt,
        _ => TokenKind::Identifier(ident.clone()),
    };

    Ok(Token::new(kind, ident, ...))
}
```

### Tokenizing Strings

```rust
fn string(&mut self) -> Result<Token, String> {
    let mut value = String::new();

    while let Some(c) = self.peek() {
        if c == '"' {
            self.advance(); // Consume closing quote
            return Ok(Token::new(
                TokenKind::String(value.clone()),
                format!("\"{}\"", value),
                ...
            ));
        } else if c == '\\' {
            // Handle escape sequences
            self.advance();
            if let Some(escaped) = self.peek() {
                self.advance();
                match escaped {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    _ => return Err(format!("Invalid escape sequence")),
                }
            }
        } else if c == '\n' {
            return Err("Unterminated string".to_string());
        } else {
            value.push(c);
            self.advance();
        }
    }

    Err("Unterminated string".to_string())
}
```

## Testing the Lexer

```rust
#[test]
fn test_integers() {
    let mut lexer = Lexer::new("42 123 0");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Integer(42));
    assert_eq!(tokens[1].kind, TokenKind::Integer(123));
    assert_eq!(tokens[2].kind, TokenKind::Integer(0));
}

#[test]
fn test_keywords() {
    let mut lexer = Lexer::new("let fn if else return");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Fn);
    assert_eq!(tokens[2].kind, TokenKind::If);
    assert_eq!(tokens[3].kind, TokenKind::Else);
    assert_eq!(tokens[4].kind, TokenKind::Return);
}

#[test]
fn test_simple_program() {
    let source = r#"
fn factorial(n: int) -> int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    // Should successfully tokenize
    assert!(tokens.len() > 20);
}
```

## Abstract Syntax Tree (AST)

After tokenization, we need to understand the **structure** of the program. This is where the AST comes in.

### What is an AST?

An **Abstract Syntax Tree** represents the syntactic structure of code as a tree.

Example:
```vos
let x = 2 + 3
```

AST:
```
Statement::Let
    name: "x"
    value: Expression::Binary
        left: Expression::Integer(2)
        op: BinaryOp::Add
        right: Expression::Integer(3)
```

### AST Node Types

**Program**: The root
```rust
pub struct Program {
    pub statements: Vec<Statement>,
}
```

**Statement**: Actions
```rust
pub enum Statement {
    Let {
        name: String,
        type_annotation: Option<Type>,
        value: Expression,
    },

    Const {
        name: String,
        value: Expression,
    },

    Function {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<Statement>,
    },

    Return(Option<Expression>),

    Expression(Expression),

    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },

    While {
        condition: Expression,
        body: Vec<Statement>,
    },
}
```

**Expression**: Values
```rust
pub enum Expression {
    Integer(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    Identifier(String),

    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },

    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },

    Call {
        function: String,
        arguments: Vec<Expression>,
    },
}
```

### Example AST

For this program:
```vos
fn add(a: int, b: int) -> int {
    return a + b
}
```

AST structure:
```
Program {
    statements: [
        Statement::Function {
            name: "add",
            params: [
                Parameter { name: "a", param_type: Type::Int },
                Parameter { name: "b", param_type: Type::Int },
            ],
            return_type: Some(Type::Int),
            body: [
                Statement::Return(
                    Some(Expression::Binary {
                        left: Box::new(Expression::Identifier("a")),
                        op: BinaryOp::Add,
                        right: Box::new(Expression::Identifier("b")),
                    })
                )
            ],
        }
    ]
}
```

## Language Grammar

The grammar defines valid syntax. Here's a simplified grammar for vos script:

```
program     → statement* EOF

statement   → let_stmt
            | const_stmt
            | fn_stmt
            | return_stmt
            | if_stmt
            | while_stmt
            | expr_stmt

let_stmt    → "let" IDENTIFIER (":" type)? "=" expression

const_stmt  → "const" IDENTIFIER "=" expression

fn_stmt     → "fn" IDENTIFIER "(" parameters ")" ("->" type)? block

return_stmt → "return" expression?

if_stmt     → "if" expression block ("else" block)?

while_stmt  → "while" expression block

expr_stmt   → expression

expression  → equality

equality    → comparison (("==" | "!=") comparison)*

comparison  → term (("<" | "<=" | ">" | ">=") term)*

term        → factor (("+" | "-") factor)*

factor      → unary (("*" | "/" | "%") unary)*

unary       → ("!" | "-") unary
            | call

call        → primary ("(" arguments ")")?

primary     → INTEGER | FLOAT | STRING | "true" | "false"
            | IDENTIFIER
            | "(" expression ")"

block       → "{" statement* "}"
```

This grammar guides the parser implementation.

## Parsing (Overview)

The **parser** reads tokens and builds the AST following the grammar.

### Recursive Descent Parsing

A common technique where each grammar rule becomes a function:

```rust
fn parse_expression(&mut self) -> Result<Expression, String> {
    self.parse_equality()
}

fn parse_equality(&mut self) -> Result<Expression, String> {
    let mut expr = self.parse_comparison()?;

    while self.match_token(&[TokenKind::EqualEqual, TokenKind::BangEqual]) {
        let op = self.previous_op();
        let right = self.parse_comparison()?;
        expr = Expression::Binary {
            left: Box::new(expr),
            op,
            right: Box::new(right),
        };
    }

    Ok(expr)
}
```

Each function handles one level of operator precedence.

## What's Next?

We've implemented:
- ✅ Token types
- ✅ Lexer (tokenizer)
- ✅ AST structure

Still to implement:
- ⏳ Parser (tokens → AST)
- ⏳ Type checker
- ⏳ Interpreter or compiler
- ⏳ Standard library

## Hands-On Exercise

Try tokenizing different programs:

```rust
use vos_lang::Lexer;

fn main() {
    let source = r#"
let x = 42
let y = x + 10
print(y)
"#;

    let mut lexer = Lexer::new(source);
    match lexer.tokenize() {
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(e) => eprintln!("Lexer error: {}", e),
    }
}
```

## Challenge Problems

1. **Add Support for Comments**: Extend the lexer to handle block comments `/* ... */`

2. **Character Literals**: Add support for character literals like `'a'`, `'\\n'`

3. **Hexadecimal Numbers**: Parse hex literals like `0xFF`, `0x10`

4. **Array Syntax**: Add tokens for arrays: `[1, 2, 3]`

5. **Lambda Syntax**: Add tokens for lambdas: `|x| x * 2`

## Key Takeaways

1. **Lexers convert text to tokens**: Smallest meaningful units
2. **ASTs represent program structure**: Trees of statements and expressions
3. **Grammar defines syntax rules**: What's valid in the language
4. **Recursive descent parsing**: Grammar rules → parsing functions
5. **Compiler pipeline has stages**: Lex → Parse → Check → Generate

## Summary

In this chapter, we designed and implemented the foundation of vos script:
- **Lexer**: Converts source code into tokens
- **AST**: Represents program structure
- **Token types**: All language elements
- **Example programs**: Demonstrate syntax

Our lexer successfully tokenizes vos script programs with:
- Integers and floats
- Strings with escape sequences
- Keywords and identifiers
- Operators and delimiters
- Comments

The AST provides a clean representation for:
- Variable declarations
- Function definitions
- Control flow structures
- Expressions with operators

With these foundations, we're ready to build the parser and interpreter in future work. The language design is complete, and the basic infrastructure is in place!

## Further Reading

- "Crafting Interpreters" by Robert Nystrom
- "Modern Compiler Implementation in ML" by Andrew Appel
- "Engineering a Compiler" by Cooper & Torczon
- Dragon Book: "Compilers: Principles, Techniques, and Tools"
- Rust's own parser: https://github.com/rust-lang/rust/tree/master/compiler/rustc_parse

Understanding how languages work gives you insight into programming itself. Every feature you use in TypeScript, Python, or Rust was designed, lexed, parsed, and compiled—just like vos script!
