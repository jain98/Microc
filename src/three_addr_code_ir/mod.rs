//! Three Address Code Intermediate representation.
//! Type checking should happen at this stage.
use std::sync::atomic::{AtomicU64, Ordering};

use derive_more::Display;

use crate::ast::ast_node::Identifier;
use crate::symbol_table::symbol::data::DataType;
use crate::symbol_table::symbol::NumType;
pub mod three_address_code;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);
static LABEL_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Represents a point in the 3AC representation
/// required to support control flow.
#[derive(Debug, Display, Copy, Clone, Eq, PartialEq, Hash)]
#[display(fmt = "label{}", _0)]
pub struct Label(u64);

impl Label {
    pub fn new() -> Self {
        Self(LABEL_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
    pub fn label(&self) -> u64 {
        self.0
    }
}

/// 3AC concept to represent int registers.
/// There is no limit to the number
/// of int temporaries that can be created.
#[derive(Debug, Copy, Clone, Display, Eq, PartialEq, Hash)]
#[display(fmt = "$T{}", _0)]
pub struct TempI(u64);

impl TempI {
    pub fn new() -> Self {
        Self(TEMP_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// 3AC concept to represent float registers.
/// There is no limit to the number
/// of int temporaries that can be created.
#[derive(Debug, Copy, Clone, Display, Eq, PartialEq, Hash)]
#[display(fmt = "$T{}", _0)]
pub struct TempF(u64);

impl TempF {
    pub fn new() -> Self {
        Self(TEMP_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Int identifier
#[derive(Debug, Display, Clone)]
pub struct IdentI(pub String);

impl From<Identifier> for IdentI {
    fn from(id: Identifier) -> Self {
        IdentI(id.to_name())
    }
}

/// Float identifier
#[derive(Debug, Display, Clone)]
pub struct IdentF(pub String);

impl From<Identifier> for IdentF {
    fn from(id: Identifier) -> Self {
        IdentF(id.to_name())
    }
}

/// String identifier
#[derive(Debug, Display, Clone)]
pub struct IdentS(pub String);

impl From<Identifier> for IdentS {
    fn from(id: Identifier) -> Self {
        IdentS(id.to_name())
    }
}

/// Represents an int type LValue
/// that can either be a temporary
/// or an int identifier.
#[derive(Debug, Clone, Display)]
pub enum LValueI {
    Temp(TempI),
    #[display(fmt = "{}", _0)]
    Id(IdentI),
}

/// Represents an float type LValue
/// that can either be a temporary
/// or an float identifier.
#[derive(Debug, Clone, Display)]
pub enum LValueF {
    Temp(TempF),
    #[display(fmt = "{}", _0)]
    Id(IdentF),
}

/// Represents a RValue that can
/// either be an int or a float
/// literal.
#[derive(Debug, Clone, Display)]
pub enum RValue {
    IntLiteral(i32),
    FloatLiteral(f64),
}

#[derive(Debug, Clone, Display)]
pub enum BinaryExprOperand {
    LValueI(LValueI),
    LValueF(LValueF),
    RValue(RValue),
}

impl BinaryExprOperand {
    pub fn is_mem_ref(&self) -> bool {
        match self {
            BinaryExprOperand::LValueI(LValueI::Id(_)) => true,
            BinaryExprOperand::LValueF(LValueF::Id(_)) => true,
            _ => false,
        }
    }
}

impl From<TempI> for BinaryExprOperand {
    fn from(temp: TempI) -> Self {
        BinaryExprOperand::LValueI(LValueI::Temp(temp))
    }
}

impl From<TempF> for BinaryExprOperand {
    fn from(temp: TempF) -> Self {
        BinaryExprOperand::LValueF(LValueF::Temp(temp))
    }
}

impl From<IdentI> for BinaryExprOperand {
    fn from(val: IdentI) -> Self {
        BinaryExprOperand::LValueI(LValueI::Id(val))
    }
}

impl From<IdentF> for BinaryExprOperand {
    fn from(val: IdentF) -> Self {
        BinaryExprOperand::LValueF(LValueF::Id(val))
    }
}

impl From<i32> for BinaryExprOperand {
    fn from(val: i32) -> Self {
        BinaryExprOperand::RValue(RValue::IntLiteral(val))
    }
}

impl From<f64> for BinaryExprOperand {
    fn from(val: f64) -> Self {
        BinaryExprOperand::RValue(RValue::FloatLiteral(val))
    }
}

impl From<LValueI> for BinaryExprOperand {
    fn from(lvalue: LValueI) -> Self {
        BinaryExprOperand::LValueI(lvalue)
    }
}

impl From<LValueF> for BinaryExprOperand {
    fn from(lvalue: LValueF) -> Self {
        BinaryExprOperand::LValueF(lvalue)
    }
}

impl From<RValue> for BinaryExprOperand {
    fn from(rvalue: RValue) -> Self {
        BinaryExprOperand::RValue(rvalue)
    }
}

// TODO: Move this to common types if there is a
//  use case outside of 3AC.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ResultType {
    Int,
    Float,
}

impl From<DataType> for ResultType {
    fn from(symbol_type: DataType) -> Self {
        match symbol_type {
            DataType::String => {
                panic!("STRING type is not a valid result of any 3AC operations.")
            }
            DataType::Num(t) => match t {
                NumType::Int => ResultType::Int,
                NumType::Float => ResultType::Float,
            },
        }
    }
}
