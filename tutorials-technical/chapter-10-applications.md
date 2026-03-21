# Chapter 10: Building Applications

## Learning Objectives

After completing this chapter, you will understand:
- Application architecture on VOS
- HTML parsing and DOM trees
- Building a simple web browser
- Writing programs in vos script
- How applications interact with the OS

## Introduction

An operating system is only as useful as the applications that run on it. In this chapter, we'll explore building applications for VOS, from simple utilities to a basic web browser.

We'll learn:
- How to write programs in vos script
- Application design patterns
- Parsing structured data (HTML)
- Rendering and user interaction

## Example Programs in vos script

Let's look at several example programs that demonstrate vos script features.

### Calculator

A simple calculator implementing basic arithmetic:

```vos
// Simple calculator program

fn add(a: int, b: int) -> int {
    return a + b
}

fn subtract(a: int, b: int) -> int {
    return a - b
}

fn multiply(a: int, b: int) -> int {
    return a * b
}

fn divide(a: int, b: int) -> int {
    if b == 0 {
        print("Error: Division by zero")
        return 0
    }
    return a / b
}

fn main() -> int {
    print("=== VOS Calculator ===")

    let x = 20
    let y = 5

    print("Add:")
    print(add(x, y))        // 25

    print("Multiply:")
    print(multiply(x, y))   // 100

    return 0
}
```

**Features demonstrated:**
- Multiple function definitions
- Parameters and return values
- Error handling (division by zero)
- Function calls

### Loops and Iteration

Demonstrating loop patterns:

```vos
fn count_to_n(n: int) {
    let i = 1
    while i <= n {
        print(i)
        i = i + 1
    }
}

fn sum_range(start: int, end: int) -> int {
    let sum = 0
    let i = start

    while i <= end {
        sum = sum + i
        i = i + 1
    }

    return sum
}

fn factorial_iterative(n: int) -> int {
    let result = 1
    let i = 2

    while i <= n {
        result = result * i
        i = i + 1
    }

    return result
}

fn main() -> int {
    count_to_n(5)                    // Prints 1 2 3 4 5

    print(sum_range(1, 10))          // 55
    print(factorial_iterative(5))    // 120

    return 0
}
```

**Features demonstrated:**
- While loops
- Loop variables
- Accumulator patterns
- Iterative algorithms

### Fibonacci

Recursive and iterative implementations:

```vos
// Recursive fibonacci
fn fibonacci_recursive(n: int) -> int {
    if n <= 1 {
        return n
    }
    return fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2)
}

// Iterative fibonacci (more efficient)
fn fibonacci_iterative(n: int) -> int {
    if n <= 1 {
        return n
    }

    let prev = 0
    let current = 1
    let i = 2

    while i <= n {
        let next = prev + current
        prev = current
        current = next
        i = i + 1
    }

    return current
}

fn main() -> int {
    let n = 10
    print("Fibonacci of")
    print(n)
    print("is")
    print(fibonacci_iterative(n))
    return 0
}
```

## Building a Web Browser

One of the most complex applications is a web browser. While modern browsers are extremely sophisticated, we can build a simple version that demonstrates core concepts.

### Browser Architecture

```
┌─────────────────────────────────────┐
│          User Input                 │
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│        HTML Parser                  │  Parse HTML text
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│      DOM Tree (Element)             │  Document structure
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│         Renderer                    │  Display content
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│          Output                     │
└─────────────────────────────────────┘
```

### HTML Elements

We represent HTML as a tree of elements:

```rust
pub enum Element {
    /// Text node
    Text(String),

    /// HTML element
    Tag {
        name: String,
        attributes: Vec<(String, String)>,
        children: Vec<Element>,
    },
}
```

Example HTML:
```html
<p>Hello <strong>World</strong></p>
```

Element tree:
```
Tag { name: "p", children: [
    Text("Hello "),
    Tag { name: "strong", children: [
        Text("World")
    ]}
]}
```

### HTML Parser Implementation

The parser converts HTML text into an element tree.

#### Parser State

```rust
pub struct HtmlParser {
    input: Vec<char>,    // Characters to parse
    pos: usize,          // Current position
}
```

#### Parsing Algorithm

```rust
pub fn parse(&mut self) -> Result<Vec<Element>, String> {
    let mut elements = Vec::new();

    while !self.is_eof() {
        self.skip_whitespace();

        if self.peek() == Some('<') {
            elements.push(self.parse_element()?);
        } else {
            elements.push(self.parse_text()?);
        }
    }

    Ok(elements)
}
```

#### Parsing an Element

```rust
fn parse_element(&mut self) -> Result<Element, String> {
    self.expect('<')?;

    // Parse tag name
    let name = self.parse_tag_name()?;

    // Parse attributes
    let attributes = self.parse_attributes()?;

    // Check for self-closing tag: <br/>
    if self.peek() == Some('/') {
        self.advance();
        self.expect('>')?;
        return Ok(Element::Tag {
            name,
            attributes,
            children: Vec::new(),
        });
    }

    self.expect('>')?;

    // Parse children until closing tag
    let mut children = Vec::new();
    loop {
        self.skip_whitespace();

        // Check for closing tag: </p>
        if self.peek() == Some('<') && self.peek_next() == Some('/') {
            self.advance(); // <
            self.advance(); // /
            let closing_name = self.parse_tag_name()?;
            self.expect('>')?;

            if closing_name != name {
                return Err(format!("Mismatched tag"));
            }
            break;
        }

        // Parse child
        if self.peek() == Some('<') {
            children.push(self.parse_element()?);
        } else {
            children.push(self.parse_text()?);
        }
    }

    Ok(Element::Tag {
        name,
        attributes,
        children,
    })
}
```

#### Parsing Attributes

```rust
fn parse_attributes(&mut self) -> Result<Vec<(String, String)>, String> {
    let mut attributes = Vec::new();

    loop {
        self.skip_whitespace();

        if self.peek() == Some('>') || self.peek() == Some('/') {
            break;
        }

        // Parse attribute name
        let name = self.parse_attribute_name()?;
        self.skip_whitespace();

        // Parse value if present
        if self.peek() == Some('=') {
            self.advance();
            self.skip_whitespace();
            let value = self.parse_attribute_value()?;
            attributes.push((name, value));
        } else {
            attributes.push((name, String::new()));
        }
    }

    Ok(attributes)
}
```

#### Parsing Attribute Values

```rust
fn parse_attribute_value(&mut self) -> Result<String, String> {
    let quote = self.peek();

    if quote != Some('"') && quote != Some('\'') {
        return Err("Expected quoted attribute value".to_string());
    }

    let quote_char = quote.unwrap();
    self.advance(); // Skip opening quote

    let mut value = String::new();

    while let Some(c) = self.peek() {
        if c == quote_char {
            self.advance(); // Skip closing quote
            return Ok(value);
        }
        value.push(c);
        self.advance();
    }

    Err("Unterminated attribute value".to_string())
}
```

### Browser Renderer

The renderer converts the element tree to displayable output.

#### Text Rendering

For a simple text-based browser:

```rust
pub struct Browser;

impl Browser {
    pub fn render(elements: &[Element]) -> String {
        let mut output = String::new();
        for element in elements {
            Self::render_element(element, &mut output, 0);
        }
        output
    }

    fn render_element(element: &Element, output: &mut String, depth: usize) {
        match element {
            Element::Text(text) => {
                if !text.is_empty() {
                    output.push_str(&"  ".repeat(depth));
                    output.push_str(text);
                    output.push('\n');
                }
            }
            Element::Tag { name, children, .. } => {
                output.push_str(&"  ".repeat(depth));
                output.push_str(&format!("<{}>\n", name));

                for child in children {
                    Self::render_element(child, output, depth + 1);
                }
            }
        }
    }
}
```

Output example:
```
<html>
  <body>
    <h1>
      Welcome to VOS
    <p>
      A simple operating system
```

#### Text Extraction

Extract just the text content:

```rust
pub fn extract_text(elements: &[Element]) -> String {
    let mut text = String::new();
    for element in elements {
        Self::extract_text_from_element(element, &mut text);
    }
    text
}

fn extract_text_from_element(element: &Element, text: &mut String) {
    match element {
        Element::Text(t) => {
            if !t.is_empty() {
                text.push_str(t);
                text.push(' ');
            }
        }
        Element::Tag { children, .. } => {
            for child in children {
                Self::extract_text_from_element(child, text);
            }
        }
    }
}
```

## Using the Browser

```rust
use vos_userspace::{HtmlParser, Browser};

fn main() {
    let html = r#"
        <html>
            <head>
                <title>VOS</title>
            </head>
            <body>
                <h1>Welcome to VOS!</h1>
                <p>A simple operating system for learning.</p>
                <ul>
                    <li>CPU emulation</li>
                    <li>Memory management</li>
                    <li>File system</li>
                </ul>
            </body>
        </html>
    "#;

    // Parse HTML
    let mut parser = HtmlParser::new(html);
    match parser.parse() {
        Ok(elements) => {
            // Render structure
            println!("=== HTML Structure ===");
            println!("{}", Browser::render(&elements));

            // Extract text
            println!("\n=== Text Content ===");
            println!("{}", Browser::extract_text(&elements));
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
```

Output:
```
=== HTML Structure ===
<html>
  <head>
    <title>
      VOS
  <body>
    <h1>
      Welcome to VOS!
    <p>
      A simple operating system for learning.
    <ul>
      <li>
        CPU emulation
      <li>
        Memory management
      <li>
        File system

=== Text Content ===
VOS Welcome to VOS! A simple operating system for learning. CPU emulation Memory management File system
```

## Testing the Browser

```rust
#[test]
fn test_parse_simple_tag() {
    let mut parser = HtmlParser::new("<p>Hello</p>");
    let elements = parser.parse().unwrap();

    assert_eq!(elements.len(), 1);
    match &elements[0] {
        Element::Tag { name, children, .. } => {
            assert_eq!(name, "p");
            assert_eq!(children.len(), 1);
        }
        _ => panic!("Expected tag"),
    }
}

#[test]
fn test_parse_with_attributes() {
    let html = r#"<a href="http://example.com">Link</a>"#;
    let mut parser = HtmlParser::new(html);
    let elements = parser.parse().unwrap();

    match &elements[0] {
        Element::Tag { name, attributes, .. } => {
            assert_eq!(name, "a");
            assert_eq!(attributes.len(), 1);
            assert_eq!(attributes[0].0, "href");
            assert_eq!(attributes[0].1, "http://example.com");
        }
        _ => panic!("Expected tag"),
    }
}

#[test]
fn test_nested_tags() {
    let html = "<html><body><h1>Title</h1><p>Text</p></body></html>";
    let mut parser = HtmlParser::new(html);
    let elements = parser.parse().unwrap();

    // Should successfully parse nested structure
    assert_eq!(elements.len(), 1);
}
```

## Application Design Patterns

### Separation of Concerns

```
┌─────────────────┐
│  Presentation   │  Display/UI
└─────────────────┘
        │
┌─────────────────┐
│    Business     │  Logic/processing
└─────────────────┘
        │
┌─────────────────┐
│      Data       │  Storage/persistence
└─────────────────┘
```

Our browser follows this:
- **Presentation**: Text renderer
- **Business**: HTML parser
- **Data**: Element tree (DOM)

### Parser Pattern

Many applications need parsers:
- **Configuration files**: JSON, TOML, YAML
- **Data formats**: CSV, XML
- **Programming languages**: vos script itself

Pattern:
1. **Lexer**: Text → Tokens
2. **Parser**: Tokens → Tree
3. **Interpreter**: Execute tree

### Error Handling

```rust
// Return Result for operations that can fail
fn parse_element(&mut self) -> Result<Element, String> {
    // Try to parse
    if let Err(e) = self.expect('<') {
        return Err(format!("Expected '<': {}", e));
    }

    // More parsing...

    Ok(element)
}

// Usage
match parser.parse() {
    Ok(elements) => {
        // Success - process elements
    }
    Err(e) => {
        // Failure - report error
        eprintln!("Parse error: {}", e);
    }
}
```

## Advanced Browser Features

Real browsers implement much more:

### CSS Styling

```rust
pub struct Style {
    pub font_size: u32,
    pub color: Color,
    pub background: Color,
}

pub fn apply_styles(element: &Element) -> Style {
    // Parse CSS
    // Match selectors
    // Apply rules
}
```

### JavaScript Execution

```rust
pub struct JavaScriptEngine {
    // VM for executing JS
}

impl JavaScriptEngine {
    pub fn execute(&mut self, code: &str) {
        // Parse JS
        // Execute in sandbox
    }
}
```

### Layout Engine

```rust
pub struct LayoutBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub children: Vec<LayoutBox>,
}

pub fn layout(element: &Element) -> LayoutBox {
    // Calculate positions
    // Handle flow, float, positioning
}
```

### Rendering

```rust
pub fn render_to_pixels(layout: &LayoutBox) -> Image {
    // Rasterize text
    // Draw boxes
    // Handle z-index
}
```

## Hands-On Exercise

Create a simple markdown renderer:

```rust
pub enum MarkdownElement {
    Heading { level: u8, text: String },
    Paragraph(String),
    CodeBlock(String),
    List(Vec<String>),
}

pub struct MarkdownParser {
    lines: Vec<String>,
}

impl MarkdownParser {
    pub fn parse(input: &str) -> Vec<MarkdownElement> {
        // Parse markdown syntax
        // Return structured elements
    }
}
```

Test it:
```markdown
# Hello VOS

This is a **simple** markdown parser.

- Feature 1
- Feature 2

`code here`
```

## Challenge Problems

1. **Add CSS Support**: Extend the browser to parse basic CSS and apply styles

2. **Markdown to HTML**: Convert markdown to HTML elements

3. **JSON Parser**: Build a JSON parser similar to the HTML parser

4. **Text Editor**: Create a simple text editor with save/load

5. **Calculator REPL**: Make the calculator interactive with a REPL

## Key Takeaways

1. **Applications layer on top of the OS**: Use kernel services via system calls
2. **Parsing is fundamental**: Many applications parse structured data
3. **Tree structures are powerful**: DOM, AST, file systems all use trees
4. **Separation of concerns**: Keep parsing, logic, and presentation separate
5. **Error handling matters**: Applications must handle invalid input gracefully

## Summary

In this chapter, we built applications for VOS:
- **Example programs in vos script**: Calculator, loops, fibonacci
- **HTML parser**: Converts HTML text to element tree
- **Simple browser**: Renders HTML structure and extracts text
- **Application patterns**: Parsing, trees, error handling

Our browser demonstrates core concepts used in real browsers:
- **Parsing**: HTML → DOM tree
- **Tree traversal**: Walking the DOM
- **Rendering**: DOM → visual output

While simplified, this shows how complex applications are built from simple components. Modern browsers have millions of lines of code, but they all start with these fundamentals!

## Further Reading

- "High Performance Browser Networking" by Ilya Grigorik
- Servo browser engine: https://servo.org/
- WebKit source: https://webkit.org/
- "How Browsers Work" by Tali Garsiel
- HTML spec: https://html.spec.whatwg.org/

Understanding how applications work helps you build better software and appreciate the complexity of tools you use every day!

## Next Steps

We've now completed the major components of VOS:
- ✅ CPU and memory
- ✅ File system
- ✅ Shell
- ✅ Language (foundation)
- ✅ Applications

In the final chapter (Chapter 11), we'll tie everything together and explore future directions for VOS!
