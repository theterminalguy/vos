//! Abstract Syntax Tree for vos script.

/// A complete program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// A statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Variable declaration: let x = 5
    Let {
        name: String,
        type_annotation: Option<Type>,
        value: Expression,
    },

    /// Constant declaration: const PI = 3.14
    Const {
        name: String,
        value: Expression,
    },

    /// Function declaration
    Function {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<Statement>,
    },

    /// Return statement
    Return(Option<Expression>),

    /// Expression statement
    Expression(Expression),

    /// If statement
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },

    /// While loop
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

/// An expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Integer literal
    Integer(i32),

    /// Float literal
    Float(f64),

    /// String literal
    String(String),

    /// Boolean literal
    Boolean(bool),

    /// Variable reference
    Identifier(String),

    /// Binary operation
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },

    /// Unary operation
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },

    /// Function call
    Call {
        function: String,
        arguments: Vec<Expression>,
    },
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical
    And,
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

/// Type annotations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Char,
}
