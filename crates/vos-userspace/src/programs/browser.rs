//! Simple web browser - HTML parser and renderer.
//!
//! A basic browser that can parse and display simple HTML.

/// HTML element types.
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    /// Text node
    Text(String),

    /// HTML element with tag, attributes, and children
    Tag {
        name: String,
        attributes: Vec<(String, String)>,
        children: Vec<Element>,
    },
}

/// HTML parser.
pub struct HtmlParser {
    input: Vec<char>,
    pos: usize,
}

impl HtmlParser {
    /// Creates a new parser.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// Parses HTML into an element tree.
    pub fn parse(&mut self) -> Result<Vec<Element>, String> {
        let mut elements = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }

            if self.peek() == Some('<') {
                elements.push(self.parse_element()?);
            } else {
                elements.push(self.parse_text()?);
            }
        }

        Ok(elements)
    }

    /// Parses an HTML element.
    fn parse_element(&mut self) -> Result<Element, String> {
        self.expect('<')?;

        // Check for closing tag
        if self.peek() == Some('/') {
            return Err("Unexpected closing tag".to_string());
        }

        // Parse tag name
        let name = self.parse_tag_name()?;

        // Parse attributes
        let attributes = self.parse_attributes()?;

        // Check for self-closing tag
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

        // Parse children
        let mut children = Vec::new();
        loop {
            self.skip_whitespace();

            if self.is_eof() {
                break;
            }

            // Check for closing tag
            if self.peek() == Some('<') && self.peek_next() == Some('/') {
                self.advance(); // <
                self.advance(); // /
                let closing_name = self.parse_tag_name()?;
                self.expect('>')?;

                if closing_name != name {
                    return Err(format!(
                        "Mismatched closing tag: expected </{}>, got </{}>",
                        name, closing_name
                    ));
                }
                break;
            }

            // Parse child element
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

    /// Parses text content.
    fn parse_text(&mut self) -> Result<Element, String> {
        let mut text = String::new();

        while !self.is_eof() {
            if self.peek() == Some('<') {
                break;
            }
            text.push(self.advance());
        }

        Ok(Element::Text(text.trim().to_string()))
    }

    /// Parses tag name.
    fn parse_tag_name(&mut self) -> Result<String, String> {
        let mut name = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if name.is_empty() {
            return Err("Expected tag name".to_string());
        }

        Ok(name)
    }

    /// Parses attributes.
    fn parse_attributes(&mut self) -> Result<Vec<(String, String)>, String> {
        let mut attributes = Vec::new();

        loop {
            self.skip_whitespace();

            if self.peek() == Some('>') || self.peek() == Some('/') {
                break;
            }

            let name = self.parse_attribute_name()?;
            self.skip_whitespace();

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

    /// Parses attribute name.
    fn parse_attribute_name(&mut self) -> Result<String, String> {
        let mut name = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if name.is_empty() {
            return Err("Expected attribute name".to_string());
        }

        Ok(name)
    }

    /// Parses attribute value.
    fn parse_attribute_value(&mut self) -> Result<String, String> {
        let quote = self.peek();

        if quote != Some('"') && quote != Some('\'') {
            return Err("Expected quoted attribute value".to_string());
        }

        let quote_char = quote.unwrap();
        self.advance();

        let mut value = String::new();

        while let Some(c) = self.peek() {
            if c == quote_char {
                self.advance();
                return Ok(value);
            }
            value.push(c);
            self.advance();
        }

        Err("Unterminated attribute value".to_string())
    }

    /// Skips whitespace.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Expects a specific character.
    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.peek() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected '{}', got {:?}",
                expected,
                self.peek()
            ))
        }
    }

    /// Advances to next character.
    fn advance(&mut self) -> char {
        let c = self.input[self.pos];
        self.pos += 1;
        c
    }

    /// Peeks at current character.
    fn peek(&self) -> Option<char> {
        if self.is_eof() {
            None
        } else {
            Some(self.input[self.pos])
        }
    }

    /// Peeks at next character.
    fn peek_next(&self) -> Option<char> {
        if self.pos + 1 >= self.input.len() {
            None
        } else {
            Some(self.input[self.pos + 1])
        }
    }

    /// Checks if at end of input.
    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

/// Browser - renders HTML to text.
pub struct Browser;

impl Browser {
    /// Renders HTML to text output.
    pub fn render(elements: &[Element]) -> String {
        let mut output = String::new();
        for element in elements {
            Self::render_element(element, &mut output, 0);
        }
        output
    }

    /// Renders a single element.
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

    /// Extracts all text from HTML.
    pub fn extract_text(elements: &[Element]) -> String {
        let mut text = String::new();
        for element in elements {
            Self::extract_text_from_element(element, &mut text);
        }
        text
    }

    /// Extracts text from a single element.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text() {
        let mut parser = HtmlParser::new("Hello World");
        let elements = parser.parse().unwrap();

        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0], Element::Text("Hello World".to_string()));
    }

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
    fn test_parse_nested_tags() {
        let html = "<html><body><h1>Title</h1><p>Text</p></body></html>";
        let mut parser = HtmlParser::new(html);
        let elements = parser.parse().unwrap();

        assert_eq!(elements.len(), 1);
    }

    #[test]
    fn test_parse_with_attributes() {
        let html = r#"<a href="http://example.com">Link</a>"#;
        let mut parser = HtmlParser::new(html);
        let elements = parser.parse().unwrap();

        match &elements[0] {
            Element::Tag {
                name, attributes, ..
            } => {
                assert_eq!(name, "a");
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].0, "href");
                assert_eq!(attributes[0].1, "http://example.com");
            }
            _ => panic!("Expected tag"),
        }
    }

    #[test]
    fn test_self_closing_tag() {
        let mut parser = HtmlParser::new("<br/>");
        let elements = parser.parse().unwrap();

        match &elements[0] {
            Element::Tag {
                name, children, ..
            } => {
                assert_eq!(name, "br");
                assert_eq!(children.len(), 0);
            }
            _ => panic!("Expected tag"),
        }
    }

    #[test]
    fn test_render() {
        let elements = vec![Element::Tag {
            name: "p".to_string(),
            attributes: vec![],
            children: vec![Element::Text("Hello".to_string())],
        }];

        let output = Browser::render(&elements);
        assert!(output.contains("<p>"));
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_extract_text() {
        let elements = vec![Element::Tag {
            name: "div".to_string(),
            attributes: vec![],
            children: vec![
                Element::Tag {
                    name: "h1".to_string(),
                    attributes: vec![],
                    children: vec![Element::Text("Title".to_string())],
                },
                Element::Tag {
                    name: "p".to_string(),
                    attributes: vec![],
                    children: vec![Element::Text("Content".to_string())],
                },
            ],
        }];

        let text = Browser::extract_text(&elements);
        assert!(text.contains("Title"));
        assert!(text.contains("Content"));
    }
}
