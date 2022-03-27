//! Tiny Assembly - https://engineering.purdue.edu/~milind/ece468/2017fall/assignments/step4/tinyDoc.txt
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use atomic_refcell::AtomicRefCell;
use derive_more::Display;

use crate::symbol_table::symbol::data::DataSymbol;
use crate::symbol_table::SymbolTable;
use crate::three_addr_code_ir;
use crate::three_addr_code_ir::three_address_code::ThreeAddressCode;
use crate::three_addr_code_ir::{BinaryExprOperand, LValueF, LValueI, RValue, TempF, TempI};
use std::rc::Rc;

static REGISTER_COUNTER: AtomicU64 = AtomicU64::new(0);

lazy_static::lazy_static! {
    static ref INT_REGISTER_MAP: AtomicRefCell<HashMap<TempI, Register>> = AtomicRefCell::new(HashMap::new());
    static ref FLOAT_REGISTER_MAP: AtomicRefCell<HashMap<TempF, Register>> = AtomicRefCell::new(HashMap::new());
}

#[derive(Debug, Copy, Clone, Display)]
#[display(fmt = "label{}", _0)]
pub struct Label(u64);

impl From<three_addr_code_ir::Label> for Label {
    fn from(label: three_addr_code_ir::Label) -> Self {
        Label(label.label())
    }
}

#[derive(Debug, Copy, Clone, Display)]
#[display(fmt = "r{}", _0)]
pub struct Register(u64);

impl Register {
    pub fn new() -> Self {
        let result = REGISTER_COUNTER.fetch_add(1, Ordering::SeqCst);
        // TODO: Add proper error type
        assert!(result < 200, "Cannot allocate more than 200 registers!");
        Self(result)
    }
}

/// Memory id, stack variable, or a register
/// https://engineering.purdue.edu/~milind/ece468/2017fall/assignments/step4/tinyDoc.txt
#[derive(Debug, Clone, Display)]
pub enum Opmr {
    Reg(Register),
    Id(Rc<DataSymbol>),
}

/// Memory id, stack variable, register or an int literal
/// https://engineering.purdue.edu/~milind/ece468/2017fall/assignments/step4/tinyDoc.txt
#[derive(Debug, Clone, Display)]
pub enum OpmrIL {
    Literal(i32),
    Location(Opmr),
}

/// Memory id, stack variable, register or a float literal
/// https://engineering.purdue.edu/~milind/ece468/2017fall/assignments/step4/tinyDoc.txt
#[derive(Debug, Clone, Display)]
pub enum OpmrFL {
    Literal(f64),
    Location(Opmr),
}

/// Memory id, stack variable, register or a number (literal)
/// https://engineering.purdue.edu/~milind/ece468/2017fall/assignments/step4/tinyDoc.txt
#[derive(Debug, Clone, Display)]
pub enum OpmrL {
    Int(OpmrIL),
    Float(OpmrFL),
}

impl OpmrL {
    fn into_int_opmrl(self) -> OpmrIL {
        match self {
            OpmrL::Int(opmrl) => opmrl,
            _ => panic!("Incorrect OpmrL variant!"),
        }
    }

    fn into_float_opmrl(self) -> OpmrFL {
        match self {
            OpmrL::Float(opmrl) => opmrl,
            _ => panic!("Incorrect OpmrL variant!"),
        }
    }
}

#[derive(Debug, Clone, Display)]
#[display(fmt = "{} {}", id, value)]
pub struct Sid {
    id: String,
    value: String,
}

#[allow(unused)]
#[derive(Debug, Display)]
pub enum TinyCode {
    #[display(fmt = "var {}", _0)]
    Var(String),
    #[display(fmt = "str {}", _0)]
    Str(Sid),
    #[display(fmt = "label {}", _0)]
    Label(Label),
    #[display(fmt = "move {} {}", _0, _1)]
    Move(OpmrL, Opmr),
    #[display(fmt = "addi {} {}", _0, _1)]
    AddI(OpmrIL, Register),
    #[display(fmt = "subi {} {}", _0, _1)]
    SubI(OpmrIL, Register),
    #[display(fmt = "muli {} {}", _0, _1)]
    MulI(OpmrIL, Register),
    #[display(fmt = "divi {} {}", _0, _1)]
    DivI(OpmrIL, Register),
    #[display(fmt = "addr {} {}", _0, _1)]
    AddF(OpmrFL, Register),
    #[display(fmt = "subr {} {}", _0, _1)]
    SubF(OpmrFL, Register),
    #[display(fmt = "mulr {} {}", _0, _1)]
    MulF(OpmrFL, Register),
    #[display(fmt = "divr {} {}", _0, _1)]
    DivF(OpmrFL, Register),
    #[display(fmt = "inci {}", _0)]
    IncI(Register),
    #[display(fmt = "deci {}", _0)]
    DecI(Register),
    #[display(fmt = "cmpi {} {}", _0, _1)]
    CmpI(OpmrIL, Register),
    #[display(fmt = "cmpr {} {}", _0, _1)]
    CmpF(OpmrFL, Register),
    #[display(fmt = "PUSH - FIXME")]
    Push(Option<OpmrL>),
    #[display(fmt = "POP - FIXME")]
    Pop(Option<Opmr>),
    #[display(fmt = "jsr {}", _0)]
    Jsr(Label),
    #[display(fmt = "RET - FIXME")]
    Ret,
    #[display(fmt = "LINK - FIXME")]
    Link(Option<u32>),
    #[display(fmt = "UNLINK - FIXME")]
    Unlink,
    #[display(fmt = "jmp {}", _0)]
    Jmp(Label),
    #[display(fmt = "jgt {}", _0)]
    Jgt(Label),
    #[display(fmt = "jlt {}", _0)]
    Jlt(Label),
    #[display(fmt = "jge {}", _0)]
    Jge(Label),
    #[display(fmt = "jle {}", _0)]
    Jle(Label),
    #[display(fmt = "jeq {}", _0)]
    Jeq(Label),
    #[display(fmt = "jne {}", _0)]
    Jne(Label),
    #[display(fmt = "sys readi {}", _0)]
    ReadI(Opmr),
    #[display(fmt = "sys readr {}", _0)]
    ReadF(Opmr),
    #[display(fmt = "sys writei {}", _0)]
    WriteI(Opmr),
    #[display(fmt = "sys writer {}", _0)]
    WriteF(Opmr),
    #[display(fmt = "sys writes {}", _0)]
    WriteS(Rc<DataSymbol>),
    #[display(fmt = "sys halt")]
    Halt,
}

impl TinyCode {
    fn binary_op_tac_operand_to_register_or_move(
        operand: BinaryExprOperand,
    ) -> (Register, Option<TinyCode>) {
        match operand {
            BinaryExprOperand::LValueI(LValueI::Temp(temp)) => {
                let register = *INT_REGISTER_MAP.borrow().get(&temp).unwrap();
                (register, None)
            }
            BinaryExprOperand::LValueF(LValueF::Temp(temp)) => {
                let register = *FLOAT_REGISTER_MAP.borrow().get(&temp).unwrap();
                (register, None)
            }
            _ => {
                let (register, move_code) =
                    TinyCode::move_binary_op_tac_operand_to_register(operand);
                (register, Some(move_code))
            }
        }
    }

    fn move_binary_op_tac_operand_to_register(operand: BinaryExprOperand) -> (Register, TinyCode) {
        match operand {
            BinaryExprOperand::LValueI(lval) => match lval {
                LValueI::Temp(temp) => {
                    let existing_reg = *INT_REGISTER_MAP.borrow().get(&temp).unwrap();
                    let new_reg = Register::new();
                    (
                        new_reg,
                        TinyCode::Move(
                            OpmrL::Int(OpmrIL::Location(Opmr::Reg(existing_reg))),
                            Opmr::Reg(new_reg),
                        ),
                    )
                }
                LValueI::Id(id) => {
                    let new_reg = Register::new();
                    (
                        new_reg,
                        TinyCode::Move(
                            OpmrL::Int(OpmrIL::Location(Opmr::Id(id.0))),
                            Opmr::Reg(new_reg),
                        ),
                    )
                }
            },
            BinaryExprOperand::LValueF(lval) => match lval {
                LValueF::Temp(temp) => {
                    let existing_reg = *FLOAT_REGISTER_MAP.borrow().get(&temp).unwrap();
                    let new_reg = Register::new();
                    (
                        new_reg,
                        TinyCode::Move(
                            OpmrL::Float(OpmrFL::Location(Opmr::Reg(existing_reg))),
                            Opmr::Reg(new_reg),
                        ),
                    )
                }
                LValueF::Id(id) => {
                    let new_reg = Register::new();
                    (
                        new_reg,
                        TinyCode::Move(
                            OpmrL::Float(OpmrFL::Location(Opmr::Id(id.0))),
                            Opmr::Reg(new_reg),
                        ),
                    )
                }
            },
            BinaryExprOperand::RValue(rval) => match rval {
                RValue::IntLiteral(n) => {
                    let new_reg = Register::new();
                    (
                        new_reg,
                        TinyCode::Move(OpmrL::Int(OpmrIL::Literal(n)), Opmr::Reg(new_reg)),
                    )
                }
                RValue::FloatLiteral(n) => {
                    let new_reg = Register::new();
                    (
                        new_reg,
                        TinyCode::Move(OpmrL::Float(OpmrFL::Literal(n)), Opmr::Reg(new_reg)),
                    )
                }
            },
        }
    }

    fn binary_op_tac_operand_to_opmrl(operand: BinaryExprOperand) -> OpmrL {
        match operand {
            BinaryExprOperand::LValueI(lval) => match lval {
                LValueI::Temp(temp) => {
                    let existing_reg = *INT_REGISTER_MAP.borrow().get(&temp).unwrap();
                    OpmrL::Int(OpmrIL::Location(Opmr::Reg(existing_reg)))
                }
                LValueI::Id(id) => OpmrL::Int(OpmrIL::Location(Opmr::Id(id.0))),
            },
            BinaryExprOperand::LValueF(lval) => match lval {
                LValueF::Temp(temp) => {
                    let existing_reg = *FLOAT_REGISTER_MAP.borrow().get(&temp).unwrap();
                    OpmrL::Float(OpmrFL::Location(Opmr::Reg(existing_reg)))
                }
                LValueF::Id(id) => OpmrL::Float(OpmrFL::Location(Opmr::Id(id.0))),
            },
            BinaryExprOperand::RValue(rval) => match rval {
                RValue::IntLiteral(n) => OpmrL::Int(OpmrIL::Literal(n)),
                RValue::FloatLiteral(n) => OpmrL::Float(OpmrFL::Literal(n)),
            },
        }
    }
}

#[derive(Debug)]
pub struct TinyCodeSequence {
    pub sequence: Vec<TinyCode>,
}

impl From<ThreeAddressCode> for TinyCodeSequence {
    fn from(three_addr_code: ThreeAddressCode) -> Self {
        match three_addr_code {
            ThreeAddressCode::AddI {
                lhs,
                rhs,
                temp_result: temporary,
            } => {
                let (operand1, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(lhs);
                let operand2 = TinyCode::binary_op_tac_operand_to_opmrl(rhs).into_int_opmrl();
                let add_code = TinyCode::AddI(operand2, operand1);

                INT_REGISTER_MAP.borrow_mut().insert(temporary, operand1);

                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(add_code);
                        result
                    },
                }
            }
            ThreeAddressCode::SubI {
                lhs,
                rhs,
                temp_result: temporary,
            } => {
                let (operand1, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(lhs);
                let operand2 = TinyCode::binary_op_tac_operand_to_opmrl(rhs).into_int_opmrl();
                let sub_code = TinyCode::SubI(operand2, operand1);

                INT_REGISTER_MAP.borrow_mut().insert(temporary, operand1);

                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(sub_code);
                        result
                    },
                }
            }
            ThreeAddressCode::MulI {
                lhs,
                rhs,
                temp_result: temporary,
            } => {
                let (operand1, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(lhs);
                let operand2 = TinyCode::binary_op_tac_operand_to_opmrl(rhs).into_int_opmrl();
                let mul_code = TinyCode::MulI(operand2, operand1);

                INT_REGISTER_MAP.borrow_mut().insert(temporary, operand1);

                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(mul_code);
                        result
                    },
                }
            }
            ThreeAddressCode::DivI {
                lhs,
                rhs,
                temp_result: temporary,
            } => {
                let (operand1, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(lhs);
                let operand2 = TinyCode::binary_op_tac_operand_to_opmrl(rhs).into_int_opmrl();
                let div_code = TinyCode::DivI(operand2, operand1);

                INT_REGISTER_MAP.borrow_mut().insert(temporary, operand1);

                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(div_code);
                        result
                    },
                }
            }
            ThreeAddressCode::StoreI { lhs, rhs } => {
                // NOTE - Only 1 of the move operands can be a memory ref.
                // The other has to be stored in a register.

                let (operand1, is_lhs_mem_ref) = match lhs {
                    LValueI::Temp(temp) => {
                        let maybe_new_register = INT_REGISTER_MAP
                            .borrow()
                            .get(&temp)
                            .copied()
                            .unwrap_or_else(Register::new);
                        INT_REGISTER_MAP
                            .borrow_mut()
                            .insert(temp, maybe_new_register);
                        (Opmr::Reg(maybe_new_register), false)
                    }
                    LValueI::Id(id) => (Opmr::Id(id.0), true),
                };

                if !is_lhs_mem_ref || !rhs.is_mem_ref() {
                    let operand2 = TinyCode::binary_op_tac_operand_to_opmrl(rhs);
                    let move_code = TinyCode::Move(operand2, operand1);
                    TinyCodeSequence {
                        sequence: vec![move_code],
                    }
                } else {
                    let (operand2, operand_move_code) =
                        TinyCode::move_binary_op_tac_operand_to_register(rhs);
                    let move_code =
                        TinyCode::Move(OpmrL::Int(OpmrIL::Location(Opmr::Reg(operand2))), operand1);
                    TinyCodeSequence {
                        sequence: vec![operand_move_code, move_code],
                    }
                }
            }
            ThreeAddressCode::ReadI { identifier } => TinyCodeSequence {
                sequence: vec![TinyCode::ReadI(Opmr::Id(identifier.0))],
            },
            ThreeAddressCode::WriteI { identifier } => TinyCodeSequence {
                sequence: vec![TinyCode::WriteI(Opmr::Id(identifier.0))],
            },
            ThreeAddressCode::AddF {
                lhs,
                rhs,
                temp_result: temporary,
            } => {
                let (operand1, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(lhs);
                let operand2 = TinyCode::binary_op_tac_operand_to_opmrl(rhs).into_float_opmrl();
                let add_code = TinyCode::AddF(operand2, operand1);

                FLOAT_REGISTER_MAP.borrow_mut().insert(temporary, operand1);

                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(add_code);
                        result
                    },
                }
            }
            ThreeAddressCode::SubF {
                lhs,
                rhs,
                temp_result: temporary,
            } => {
                let (operand1, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(lhs);
                let operand2 = TinyCode::binary_op_tac_operand_to_opmrl(rhs).into_float_opmrl();
                let sub_code = TinyCode::SubF(operand2, operand1);

                FLOAT_REGISTER_MAP.borrow_mut().insert(temporary, operand1);

                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(sub_code);
                        result
                    },
                }
            }
            ThreeAddressCode::MulF {
                lhs,
                rhs,
                temp_result: temporary,
            } => {
                let (operand1, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(lhs);
                let operand2 = TinyCode::binary_op_tac_operand_to_opmrl(rhs).into_float_opmrl();
                let mul_code = TinyCode::MulF(operand2, operand1);

                FLOAT_REGISTER_MAP.borrow_mut().insert(temporary, operand1);

                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(mul_code);
                        result
                    },
                }
            }
            ThreeAddressCode::DivF {
                lhs,
                rhs,
                temp_result: temporary,
            } => {
                let (operand1, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(lhs);
                let operand2 = TinyCode::binary_op_tac_operand_to_opmrl(rhs).into_float_opmrl();
                let div_code = TinyCode::DivF(operand2, operand1);

                FLOAT_REGISTER_MAP.borrow_mut().insert(temporary, operand1);

                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(div_code);
                        result
                    },
                }
            }
            ThreeAddressCode::StoreF { lhs, rhs } => {
                let (operand1, is_lhs_mem_ref) = match lhs {
                    LValueF::Temp(temp) => {
                        let maybe_new_register = FLOAT_REGISTER_MAP
                            .borrow()
                            .get(&temp)
                            .copied()
                            .unwrap_or_else(Register::new);
                        FLOAT_REGISTER_MAP
                            .borrow_mut()
                            .insert(temp, maybe_new_register);
                        (Opmr::Reg(maybe_new_register), false)
                    }
                    LValueF::Id(id) => (Opmr::Id(id.0), true),
                };

                if !is_lhs_mem_ref || !rhs.is_mem_ref() {
                    let operand2 = TinyCode::binary_op_tac_operand_to_opmrl(rhs);
                    let move_code = TinyCode::Move(operand2, operand1);
                    TinyCodeSequence {
                        sequence: vec![move_code],
                    }
                } else {
                    let (operand2, operand_move_code) =
                        TinyCode::move_binary_op_tac_operand_to_register(rhs);
                    let move_code = TinyCode::Move(
                        OpmrL::Float(OpmrFL::Location(Opmr::Reg(operand2))),
                        operand1,
                    );
                    TinyCodeSequence {
                        sequence: vec![operand_move_code, move_code],
                    }
                }
            }
            ThreeAddressCode::ReadF { identifier } => TinyCodeSequence {
                sequence: vec![TinyCode::ReadF(Opmr::Id(identifier.0))],
            },
            ThreeAddressCode::WriteF { identifier } => TinyCodeSequence {
                sequence: vec![TinyCode::WriteF(Opmr::Id(identifier.0))],
            },
            ThreeAddressCode::WriteS { identifier } => TinyCodeSequence {
                sequence: vec![TinyCode::WriteS(identifier.0)],
            },
            ThreeAddressCode::Label(label) => TinyCodeSequence {
                sequence: vec![TinyCode::Label(label.into())],
            },
            ThreeAddressCode::Jump(label) => TinyCodeSequence {
                sequence: vec![TinyCode::Jmp(label.into())],
            },
            ThreeAddressCode::GtI { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpI(operand1.into_int_opmrl(), operand2);
                let jump_code = TinyCode::Jgt(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::LtI { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpI(operand1.into_int_opmrl(), operand2);
                let jump_code = TinyCode::Jlt(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::GteI { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpI(operand1.into_int_opmrl(), operand2);
                let jump_code = TinyCode::Jge(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::LteI { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpI(operand1.into_int_opmrl(), operand2);
                let jump_code = TinyCode::Jle(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::NeI { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpI(operand1.into_int_opmrl(), operand2);
                let jump_code = TinyCode::Jne(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::EqI { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpI(operand1.into_int_opmrl(), operand2);
                let jump_code = TinyCode::Jeq(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::GtF { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpF(operand1.into_float_opmrl(), operand2);
                let jump_code = TinyCode::Jgt(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::LtF { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpF(operand1.into_float_opmrl(), operand2);
                let jump_code = TinyCode::Jlt(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::GteF { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpF(operand1.into_float_opmrl(), operand2);
                let jump_code = TinyCode::Jge(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::LteF { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpF(operand1.into_float_opmrl(), operand2);
                let jump_code = TinyCode::Jle(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::NeF { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpF(operand1.into_float_opmrl(), operand2);
                let jump_code = TinyCode::Jne(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
            ThreeAddressCode::EqF { lhs, rhs, label } => {
                let operand1 = TinyCode::binary_op_tac_operand_to_opmrl(lhs);
                let (operand2, move_code) =
                    TinyCode::binary_op_tac_operand_to_register_or_move(rhs);

                let cmp_code = TinyCode::CmpF(operand1.into_float_opmrl(), operand2);
                let jump_code = TinyCode::Jeq(label.into());
                TinyCodeSequence {
                    sequence: {
                        let mut result = move_code.map_or(vec![], |move_code| vec![move_code]);
                        result.push(cmp_code);
                        result.push(jump_code);
                        result
                    },
                }
            }
        }
    }
}

impl From<Vec<ThreeAddressCode>> for TinyCodeSequence {
    fn from(three_adr_code_seq: Vec<ThreeAddressCode>) -> Self {
        // // Add all symbol declarations to tiny code sequence
        // let symbol_decls = SymbolTable::seal()
        //     .into_iter()
        //     .map(|symbol| match symbol {
        //         DataSymbol::String { name, value } => TinyCode::Str(Sid { id: name, value }),
        //         DataSymbol::Int { name } => TinyCode::Var(name),
        //         DataSymbol::Float { name } => TinyCode::Var(name),
        //     })
        //     .collect();
        //
        // let mut result = TinyCodeSequence {
        //     sequence: symbol_decls,
        // };
        //
        // result.sequence.extend(
        //     three_adr_code_seq
        //         .into_iter()
        //         .flat_map(|code| Into::<TinyCodeSequence>::into(code).sequence),
        // );
        //
        // result.sequence.push(TinyCode::Halt);

        TinyCodeSequence {
            sequence: vec![], // result.sequence,
        }
    }
}
