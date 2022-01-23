use crate::symbol_table::symbol::data::{DataSymbol, DataType};
use crate::symbol_table::symbol::NumType;
use std::rc::Rc;

/// Differentiates an addition `Add` node
/// from a subtraction `Add` node.
#[derive(Debug, Copy, Clone)]
pub enum AddOp {
    Add,
    Sub,
}

/// Differentiates an multiplication
/// `Mul` node from a division
/// `Mul` node.
#[derive(Debug, Copy, Clone)]
pub enum MulOp {
    Mul,
    Div,
}

/// Represents the comparison
/// operation in a boolean expression.
#[derive(Debug, Copy, Clone)]
pub enum CmpOp {
    /// Less than
    Lt,
    /// Greater than
    Gt,
    /// Equal to
    Eq,
    /// Not equal to
    Ne,
    /// Less than or equal to
    Lte,
    /// Greater than or equal to
    Gte,
}

/// Represents an identifier
/// for a declared data symbol.
#[derive(Debug, Clone)]
pub struct Identifier {
    pub symbol: Rc<DataSymbol>,
}

impl Identifier {
    pub fn data_type(&self) -> DataType {
        match *self.symbol {
            DataSymbol::String { .. } => DataType::String,
            DataSymbol::Int { .. } => DataType::Num(NumType::Int),
            DataSymbol::Float { .. } => DataType::Num(NumType::Float),
        }
    }

    // TODO: Needs to go away.
    //  The `IdentI`, `IdentF` and `IdentS`
    //  structs in 3AC can most likely just
    //  store a `Rc<DataSymbol>`.
    pub fn to_name(&self) -> String {
        self.symbol.name().to_owned()
    }
}

/// Math expressions in Microc
/// that evaluate to a numeric
/// value.
#[derive(Debug, Clone)]
pub enum Expr {
    Id(Identifier),
    IntLiteral(i32),
    FloatLiteral(f64),
    Add {
        op: AddOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Mul {
        op: MulOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    None,
}

/// An assignment, which exists only
/// for building different statements
/// made up of assign semantics, such as,
/// assign, if and for statements.
#[derive(Debug, Clone)]
pub struct Assignment {
    pub lhs: Identifier,
    pub rhs: Expr,
}

/// A boolean expression that evaluates
/// to either true or false.
#[derive(Debug, Clone)]
pub struct Condition {
    pub cmp_op: CmpOp,
    pub lhs: Expr,
    pub rhs: Expr,
}

/// Statements in Microc.
#[derive(Debug, Clone)]
pub enum Stmt {
    Read(Vec<Identifier>),
    Write(Vec<Identifier>),
    Assign(Assignment),
    If {
        condition: Condition,
        then_block: Vec<Stmt>,
        else_block: Vec<Stmt>,
    },
    For {
        init: Option<Assignment>,
        condition: Condition,
        incr: Option<Assignment>,
        body: Vec<Stmt>,
    },
}

/// Represents possible return types
/// in a function.
#[derive(Debug, Clone, Copy)]
pub enum FunctionReturnType {
    Int,
    Float,
    Void,
}

/// Represents constructs in Microc
/// that can be composed from expressions
/// and statements. Currently, the only
/// such valid construct in Microc is
/// functions. But this can change in the
/// future to support classes/structs/enums etc.
#[derive(Debug, Clone)]
pub enum Item {
    Function {
        name: String,
        return_type: FunctionReturnType,
        body: Vec<Stmt>,
    },
}

/// Abstract syntax tree representation
/// for Microc.
#[derive(Debug)]
pub enum AstNode {
    Item(Item),
    Stmt(Stmt),
    Expr(Expr),
}

pub mod visit {
    use super::*;

    /// Visitor trait that must be implemented
    /// by different intermediate representations
    /// that can be generated from the AST
    /// representation of Microc.
    pub trait Visitor<T> {
        // fn visit_item(&mut self, item: Item) -> T;
        fn visit_statement(&mut self, stmt: Stmt) -> T;
        fn visit_expression(&mut self, expr: Expr) -> T;
        fn visit_assignment(&mut self, assigment: Assignment) -> T;
        fn visit_condition(&mut self, condition: Condition) -> T;
    }
}
