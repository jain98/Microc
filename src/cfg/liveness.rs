use linked_hash_map::LinkedHashMap;
use crate::cfg::basic_block::{BBLabel, ImmutableBasicBlock, is_bb_terminator};
use crate::three_addr_code_ir::{LValueI, LValueF, LValue, BinaryExprOperandI, BinaryExprOperandF, IdentI, IdentF};
use crate::three_addr_code_ir::three_address_code::ThreeAddressCode;
use std::collections::HashSet;
use crate::cfg::ControlFlowGraph;
use crate::symbol_table::SymbolTable;
use crate::symbol_table::symbol::data::DataType;
use crate::symbol_table::symbol::{NumType, data};
use std::fmt::{Display, Formatter};

/// ThreeAddressCode nodes containing GEN, KILL, IN
/// and OUT sets for the current 3AC node.
#[derive(Debug, PartialEq)]
pub struct LivenessDecoratedThreeAddressCode {
    tac: ThreeAddressCode,
    gen_set: HashSet<LValue>,
    kill_set: HashSet<LValue>,
    in_set: HashSet<LValue>,
    out_set: HashSet<LValue>,
}

impl LivenessDecoratedThreeAddressCode {
    pub fn gen_set(&self) -> impl Iterator<Item=&LValue> {
        self.gen_set.iter()
    }

    pub fn kill_set(&self) -> impl Iterator<Item=&LValue> {
        self.kill_set.iter()
    }

    pub fn in_set(&self) -> impl Iterator<Item=&LValue> {
        self.in_set.iter()
    }

    pub fn out_set(&self) -> impl Iterator<Item=&LValue> {
        self.out_set.iter()
    }

    pub fn tac(&self) -> &ThreeAddressCode {
        &self.tac
    }
}

impl From<ThreeAddressCode> for LivenessDecoratedThreeAddressCode {
    fn from(tac: ThreeAddressCode) -> Self {
        let mut gen_set = HashSet::new();
        let mut kill_set = HashSet::new();
        let mut out_set = HashSet::new();

        match &tac {
            ThreeAddressCode::AddI { lhs, rhs, temp_result } |
            ThreeAddressCode::SubI { lhs, rhs, temp_result } |
            ThreeAddressCode::MulI { lhs, rhs, temp_result } |
            ThreeAddressCode::DivI { lhs, rhs, temp_result } => {
                if let BinaryExprOperandI::LValue(lvalue) = lhs {
                    gen_set.insert(LValue::LValueI(lvalue.clone()));
                }

                if let BinaryExprOperandI::LValue(lvalue) = rhs {
                    gen_set.insert(LValue::LValueI(lvalue.clone()));
                }

                kill_set.insert((*temp_result).into());
            },
            ThreeAddressCode::StoreI { lhs, rhs} => {
                if let BinaryExprOperandI::LValue(lvalue) = rhs {
                    gen_set.insert(LValue::LValueI(lvalue.clone()));
                }

                kill_set.insert(LValue::LValueI(lhs.clone()));
            }
            ThreeAddressCode::ReadI { identifier } => {
                kill_set.insert(LValue::LValueI(LValueI::Id(identifier.clone())));
            }
            ThreeAddressCode::WriteI { identifier } => {
                gen_set.insert(LValue::LValueI(LValueI::Id(identifier.clone())));
            }
            ThreeAddressCode::AddF { lhs, rhs, temp_result } |
            ThreeAddressCode::SubF { lhs, rhs, temp_result } |
            ThreeAddressCode::MulF { lhs, rhs, temp_result } |
            ThreeAddressCode::DivF { lhs, rhs, temp_result } => {
                if let BinaryExprOperandF::LValue(lvalue) = lhs {
                    gen_set.insert(LValue::LValueF(lvalue.clone()));
                }

                if let BinaryExprOperandF::LValue(lvalue) = rhs {
                    gen_set.insert(LValue::LValueF(lvalue.clone()));
                }

                kill_set.insert((*temp_result).into());
            }
            ThreeAddressCode::StoreF { lhs, rhs } => {
                if let BinaryExprOperandF::LValue(lvalue) = rhs {
                    gen_set.insert(LValue::LValueF(lvalue.clone()));
                }

                kill_set.insert(LValue::LValueF(lhs.clone()));
            }
            ThreeAddressCode::ReadF { identifier } => {
                kill_set.insert(LValue::LValueF(LValueF::Id(identifier.clone())));
            }
            ThreeAddressCode::WriteF { identifier } => {
                gen_set.insert(LValue::LValueF(LValueF::Id(identifier.clone())));
            }
            ThreeAddressCode::GtI { lhs, rhs, .. } |
            ThreeAddressCode::LtI { lhs, rhs, .. } |
            ThreeAddressCode::GteI { lhs, rhs, .. } |
            ThreeAddressCode::LteI { lhs, rhs, .. } |
            ThreeAddressCode::NeI { lhs, rhs, .. } |
            ThreeAddressCode::EqI { lhs, rhs, .. } => {
                if let BinaryExprOperandI::LValue(lvalue) = lhs {
                    gen_set.insert(LValue::LValueI(lvalue.clone()));
                }

                if let BinaryExprOperandI::LValue(lvalue) = rhs {
                    gen_set.insert(LValue::LValueI(lvalue.clone()));
                }
            }
            ThreeAddressCode::GtF { lhs, rhs, .. } |
            ThreeAddressCode::LtF { lhs, rhs, .. } |
            ThreeAddressCode::GteF { lhs, rhs, .. } |
            ThreeAddressCode::LteF { lhs, rhs, .. } |
            ThreeAddressCode::NeF { lhs, rhs, .. } |
            ThreeAddressCode::EqF { lhs, rhs, .. } => {
                if let BinaryExprOperandF::LValue(lvalue) = lhs {
                    gen_set.insert(LValue::LValueF(lvalue.clone()));
                }

                if let BinaryExprOperandF::LValue(lvalue) = rhs {
                    gen_set.insert(LValue::LValueF(lvalue.clone()));
                }
            }
            ThreeAddressCode::PushI(op) => {
                if let BinaryExprOperandI::LValue(lvalue) = op {
                    gen_set.insert(LValue::LValueI(lvalue.clone()));
                }
            }
            ThreeAddressCode::PushF(op) => {
                if let BinaryExprOperandF::LValue(lvalue) = op {
                    gen_set.insert(LValue::LValueF(lvalue.clone()));
                }
            }
            ThreeAddressCode::PopI(op) => {
                kill_set.insert(LValue::LValueI(op.clone()));
            }
            ThreeAddressCode::PopF(op) => {
                kill_set.insert(LValue::LValueF(op.clone()));
            }
            ThreeAddressCode::Jsr(_) => {
                SymbolTable::global_symbols()
                    .into_iter()
                    .filter_map(|symbol| match symbol.data_type() {
                        DataType::Num(NumType::Int) => Some(LValue::LValueI(LValueI::Id(IdentI(symbol.into())))),
                        DataType::Num(NumType::Float) => Some(LValue::LValueF(LValueF::Id(IdentF(symbol.into())))),
                        _ => None
                    })
                    .for_each(|symbol| {
                        gen_set.insert(symbol);
                    });
            }
            // `Ret` 3AC is an exceptional instruction for which we don't add
            // any gen and use sets, but add the out set, which is always all
            // the globals present in the program because global variables may
            // be used after the function returns.
            ThreeAddressCode::Ret => {
                SymbolTable::global_symbols()
                    .into_iter()
                    .filter_map(|symbol| match symbol.data_type() {
                        DataType::Num(NumType::Int) => Some(LValue::LValueI(LValueI::Id(IdentI(symbol.into())))),
                        DataType::Num(NumType::Float) => Some(LValue::LValueF(LValueF::Id(IdentF(symbol.into())))),
                        _ => None
                    })
                    .for_each(|symbol| {
                        out_set.insert(symbol);
                    });
            }
            _ => ()
        }

        LivenessDecoratedThreeAddressCode {
            tac,
            gen_set,
            kill_set,
            in_set: HashSet::new(),
            out_set,
        }
    }
}

impl Display for LivenessDecoratedThreeAddressCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tac())?;
        write!(f, "     | GEN: ")?;
        self.gen_set().try_for_each(|x| write!(f, "{x} "))?;
        write!(f, "     | KILL: ")?;
        self.kill_set().try_for_each(|x| write!(f, "{x} "))
    }
}

/// Immutable basic block containing a sequence of
/// `LivenessDecoratedThreeAddressCode` nodes.
#[derive(Debug, PartialEq)]
pub struct LivenessDecoratedImmutableBasicBlock {
    label: BBLabel,
    seq: Vec<LivenessDecoratedThreeAddressCode>,

}

impl LivenessDecoratedImmutableBasicBlock {
    pub fn label(&self) -> BBLabel {
        self.label
    }

    pub fn seq(&self) -> &[LivenessDecoratedThreeAddressCode] {
        &self.seq
    }

    pub fn seq_mut(&mut self) -> &mut [LivenessDecoratedThreeAddressCode] {
        &mut self.seq
    }

    fn first(&self) -> &LivenessDecoratedThreeAddressCode {
        // A basic block is guaranteed to never be empty
        &self.seq[0]
    }

    fn last(&self) -> &LivenessDecoratedThreeAddressCode {
        // A basic block is guaranteed to never be empty
        &self.seq[self.seq.len()-1]
    }

    pub fn in_set(&self) -> impl Iterator<Item = &LValue> {
        self.first().in_set()
    }

    pub fn out_set(&self) -> impl Iterator<Item = &LValue> {
        self.last().out_set()
    }
}

impl From<ImmutableBasicBlock> for LivenessDecoratedImmutableBasicBlock {
    fn from(bb: ImmutableBasicBlock) -> Self {
        let (label, seq) = bb.into_parts();
        Self {
            label,
            seq: seq.into_iter()
                .map(|tac| Into::<LivenessDecoratedThreeAddressCode>::into(tac))
                .collect()
        }
    }
}

impl Display for LivenessDecoratedImmutableBasicBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.label())?;
        for tac in self.seq() {
            writeln!(f, "{}", tac)?;
        }

        Ok(())
    }
}

/// Control flow graph containing `LivenessDecoratedImmutableBasicBlock`s.
#[derive(Debug, PartialEq)]
pub struct LivenessDecoratedControlFlowGraph {
    /// Basic block map - maps parent basic block
    /// to child basic blocks.
    bb_map: LinkedHashMap<BBLabel, Vec<BBLabel>>,
    /// List of basic blocks contained in the control
    /// flow graph.
    bbs: LinkedHashMap<BBLabel, LivenessDecoratedImmutableBasicBlock>,
}

impl LivenessDecoratedControlFlowGraph {
    pub fn basic_blocks(&self) -> impl Iterator<Item = (&BBLabel, &LivenessDecoratedImmutableBasicBlock)> {
        self.bbs.iter()
    }

    pub fn basic_blocks_mut(&mut self) -> impl Iterator<Item = (&BBLabel, &mut LivenessDecoratedImmutableBasicBlock)> {
        self.bbs.iter_mut()
    }

    pub fn basic_block_map(&self) -> impl Iterator<Item = (&BBLabel, &Vec<BBLabel>)> {
        self.bb_map.iter()
    }

    pub fn basic_block_for_label(&self, bb_label: &BBLabel) -> Option<&LivenessDecoratedImmutableBasicBlock> {
        self.bbs.get(bb_label)
    }

    pub fn neighbors_of_bb(&self, bb_label: &BBLabel) -> Option<&[BBLabel]> {
        self.bb_map.get(bb_label).map(|neighbors| neighbors.as_slice())
    }

    /// Updates the in and out sets associated to each 3AC node
    /// present in the CFG's basic blocks.
    pub fn update_in_and_out_sets(&mut self) {
        /*
        1. Put all of the IR nodes on the worklist
        2. Pull an IR node off the worklist, and compute its live-out and live-in sets according to the definitions above.
        3. If the live-in set of the node gets updated by the previous step, put all of the node's predecessors on the worklist (because they may need to update their live-out sets).
        4. Repeat steps 2 and 3 until the worklist is empty.
        */
        loop {
            let mut worklist: Vec<(BBLabel, &mut LivenessDecoratedThreeAddressCode)> = self.basic_blocks_mut()
                .flat_map(|(bb_label, bb)| bb.seq_mut()
                    .iter_mut()
                    .map(move |tac| (*bb_label, tac))
                )
                .collect();

            // We cannot reverse the iterator before collecting worklist
            // items into a Vec because an iterator needs to implement
            // `DoubleEndedIterator` in order for it to be reversed. An
            // iterator created from a `LinkedHashMap` does not implement
            // the `DoubleEndedIterator` trait.
            worklist.reverse();

            let mut updated = false;
            let mut successor_tac_node_in_set = HashSet::new();

            for (bb_label, tac) in worklist {
                // Find current nodes successors. Two things -
                // 1. Since we are iterating the 3AC instructions fot the
                // function in the reverse direction, the current node's successor
                // is stored in the `prev` variable declared above.
                //
                // 2. If this is not a bb terminator, then `prev` may be the only successor
                // for the current node. Otherwise, the node will have successors in addition
                // to or other than `prev`.
                let mut out_set = HashSet::new();

                // If the current 3AC is not an unconditional jump then the
                // successor 3AC node's (which we actually visited in the previous
                // loop pass) in_set is part of the current 3AC node's out_set.
                if !tac.tac().is_unconditional_branch() {
                    out_set.extend(successor_tac_node_in_set);
                }

                // If this is a bb terminator then this 3AC node is
                // going to have other successors that are the leaders
                // of the children bbs.
                if is_bb_terminator(tac.tac()) {
                    if let Some(neighbors_of_bb) =  self.neighbors_of_bb(&bb_label) {
                        for neighboring_bb in neighbors_of_bb {
                            if let Some(neighbor) = self.basic_block_for_label(neighboring_bb) {
                                out_set.extend(neighbor.in_set().cloned())
                            }
                        }
                    }
                }

                // let in_set = (tac.out_set - tac.kill_set) U tac.gen_set
                // if in_set != tac.in_set => updated = true
                // Update previous
                successor_tac_node_in_set = HashSet::new();
            }

            if !updated {
                break;
            }
        }
    }
}

impl From<ControlFlowGraph> for LivenessDecoratedControlFlowGraph {
    fn from(cfg: ControlFlowGraph) -> Self {
        let (bb_map, bbs) = cfg.into_parts();
        Self {
            bb_map,
            bbs: bbs.into_iter()
                .map(|(bb_label, bb)| (bb_label, Into::<LivenessDecoratedImmutableBasicBlock>::into(bb)))
                .collect(),
        }
    }
}

impl Display for LivenessDecoratedControlFlowGraph {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "==== Basic Blocks ===")?;
        for (_, bb) in self.basic_blocks() {
            writeln!(f, "{}", bb)?;
        }

        writeln!(f, "==== CFG ===")?;
        for (from, to) in self.basic_block_map() {
            writeln!(f, "{}: {:?}", from, to)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cfg::basic_block::{ImmutableBasicBlock, BBLabel};
    use crate::three_addr_code_ir::{IdentI, TempI, BinaryExprOperandI, LValueI, FunctionIdent};
    use crate::symbol_table::symbol::{data, function};
    use std::rc::Rc;
    use crate::three_addr_code_ir::three_address_code::ThreeAddressCode;
    use crate::cfg::liveness::{LivenessDecoratedImmutableBasicBlock, LivenessDecoratedThreeAddressCode, LValue};
    use std::collections::HashSet;
    use crate::symbol_table::{symbol_table_test_setup, SymbolTable};
    use crate::symbol_table::symbol::function::ReturnType;
    use serial_test::serial;

    #[test]
    fn push_instruction_gens_var_being_pushed() {
        let a = IdentI(data::Symbol::NonFunctionScopedSymbol(Rc::new(
            data::NonFunctionScopedSymbol::Int {
                name: "A".to_owned(),
            },
        )));

        let bb_label: BBLabel = 0.into();

        let seq = vec![
            ThreeAddressCode::PushI (BinaryExprOperandI::LValue(LValueI::Id(a.clone()))),
        ];

        let immutable_bb: ImmutableBasicBlock = (bb_label, seq).into();

        // Expected `GenKillDecoratedImmutableBasicBlock`
        let expected_gen_kill_decorated_bb = LivenessDecoratedImmutableBasicBlock {
            label: immutable_bb.label(),
            seq: vec![
                LivenessDecoratedThreeAddressCode {
                    tac: ThreeAddressCode::PushI (BinaryExprOperandI::LValue(LValueI::Id(a.clone()))),
                    gen_set: {
                        let mut gen = HashSet::new();
                        gen.insert(LValue::LValueI(LValueI::Id(a.clone())));
                        gen
                    },
                    kill_set: HashSet::new(),
                    in_set: HashSet::new(),
                    out_set: HashSet::new(),
                },
            ]
        };

        // Actual `GenKillDecoratedImmutableBasicBlock`
        let actual_gen_kill_decorated_bb: LivenessDecoratedImmutableBasicBlock = immutable_bb.into();
        assert_eq!(expected_gen_kill_decorated_bb, actual_gen_kill_decorated_bb);
    }

    #[test]
    fn pop_instruction_kills_var_being_pushed() {
        let a = IdentI(data::Symbol::NonFunctionScopedSymbol(Rc::new(
            data::NonFunctionScopedSymbol::Int {
                name: "A".to_owned(),
            },
        )));

        let bb_label: BBLabel = 0.into();

        let seq = vec![
            ThreeAddressCode::PopI(LValueI::Id(a.clone())),
        ];

        let immutable_bb: ImmutableBasicBlock = (bb_label, seq).into();

        // Expected `GenKillDecoratedImmutableBasicBlock`
        let expected_gen_kill_decorated_bb = LivenessDecoratedImmutableBasicBlock {
            label: immutable_bb.label(),
            seq: vec![
                LivenessDecoratedThreeAddressCode {
                    tac: ThreeAddressCode::PopI(LValueI::Id(a.clone())),
                    gen_set: HashSet::new(),
                    kill_set: {
                        let mut kill = HashSet::new();
                        kill.insert(LValue::LValueI(LValueI::Id(a.clone())));
                        kill
                    },
                    in_set: HashSet::new(),
                    out_set: HashSet::new(),
                },
            ]
        };

        // Actual `GenKillDecoratedImmutableBasicBlock`
        let actual_gen_kill_decorated_bb: LivenessDecoratedImmutableBasicBlock = immutable_bb.into();
        assert_eq!(expected_gen_kill_decorated_bb, actual_gen_kill_decorated_bb);
    }

    #[test]
    fn write_instruction_kills_var_being_pushed() {
        let a = IdentI(data::Symbol::NonFunctionScopedSymbol(Rc::new(
            data::NonFunctionScopedSymbol::Int {
                name: "A".to_owned(),
            },
        )));

        let bb_label: BBLabel = 0.into();

        let seq = vec![
            ThreeAddressCode::WriteI {
                identifier: a.clone(),
            },
        ];

        let immutable_bb: ImmutableBasicBlock = (bb_label, seq).into();

        // Expected `GenKillDecoratedImmutableBasicBlock`
        let expected_gen_kill_decorated_bb = LivenessDecoratedImmutableBasicBlock {
            label: immutable_bb.label(),
            seq: vec![
                LivenessDecoratedThreeAddressCode {
                    tac: ThreeAddressCode::WriteI {
                        identifier: a.clone(),
                    },
                    gen_set: {
                        let mut gen = HashSet::new();
                        gen.insert(LValue::LValueI(LValueI::Id(a.clone())));
                        gen
                    },
                    kill_set: HashSet::new(),
                    in_set: HashSet::new(),
                    out_set: HashSet::new(),
                },
            ]
        };

        // Actual `GenKillDecoratedImmutableBasicBlock`
        let actual_gen_kill_decorated_bb: LivenessDecoratedImmutableBasicBlock = immutable_bb.into();
        assert_eq!(expected_gen_kill_decorated_bb, actual_gen_kill_decorated_bb);
    }

    #[test]
    fn read_instruction_kills_var_being_pushed() {
        let a = IdentI(data::Symbol::NonFunctionScopedSymbol(Rc::new(
            data::NonFunctionScopedSymbol::Int {
                name: "A".to_owned(),
            },
        )));

        let bb_label: BBLabel = 0.into();

        let seq = vec![
            ThreeAddressCode::ReadI {
                identifier: a.clone(),
            }
        ];

        let immutable_bb: ImmutableBasicBlock = (bb_label, seq).into();

        // Expected `GenKillDecoratedImmutableBasicBlock`
        let expected_gen_kill_decorated_bb = LivenessDecoratedImmutableBasicBlock {
            label: immutable_bb.label(),
            seq: vec![
                LivenessDecoratedThreeAddressCode {
                    tac: ThreeAddressCode::ReadI {
                        identifier: a.clone(),
                    },
                    gen_set: HashSet::new(),
                    kill_set: {
                        let mut kill = HashSet::new();
                        kill.insert(LValue::LValueI(LValueI::Id(a.clone())));
                        kill
                    },
                    in_set: HashSet::new(),
                    out_set: HashSet::new(),
                },
            ]
        };

        // Actual `GenKillDecoratedImmutableBasicBlock`
        let actual_gen_kill_decorated_bb: LivenessDecoratedImmutableBasicBlock = immutable_bb.into();
        assert_eq!(expected_gen_kill_decorated_bb, actual_gen_kill_decorated_bb);
    }

    #[test]
    #[serial]
    fn call_instruction_gens_all_globals() {
        symbol_table_test_setup();

        let a = data::NonFunctionScopedSymbol::Int {
            name: "A".to_owned(),
        };
        let b = data::NonFunctionScopedSymbol::Int {
            name: "B".to_owned(),
        };
        let c = data::NonFunctionScopedSymbol::Int {
            name: "C".to_owned(),
        };

        SymbolTable::add_non_func_scoped_symbol(a.clone()).unwrap();
        SymbolTable::add_non_func_scoped_symbol(b.clone()).unwrap();
        SymbolTable::add_non_func_scoped_symbol(c.clone()).unwrap();

        let function_ident =
            FunctionIdent(Rc::new(function::Symbol::new("some_func".to_owned(), ReturnType::Void, vec![], vec![])));

        let seq = vec![
            ThreeAddressCode::Jsr(function_ident.clone()),
        ];

        let bb_label: BBLabel = 0.into();

        let immutable_bb: ImmutableBasicBlock = (bb_label, seq).into();

        // Expected `GenKillDecoratedImmutableBasicBlock`
        let expected_gen_kill_decorated_bb = LivenessDecoratedImmutableBasicBlock {
            label: immutable_bb.label(),
            seq: vec![
                LivenessDecoratedThreeAddressCode {
                    tac: ThreeAddressCode::Jsr(function_ident.clone()),
                    gen_set: {
                        let mut gen = HashSet::new();
                        gen.insert(LValue::LValueI(LValueI::Id(IdentI(Rc::new(a.clone()).into()))));
                        gen.insert(LValue::LValueI(LValueI::Id(IdentI(Rc::new(b.clone()).into()))));
                        gen.insert(LValue::LValueI(LValueI::Id(IdentI(Rc::new(c.clone()).into()))));
                        gen
                    },
                    kill_set: HashSet::new(),
                    in_set: HashSet::new(),
                    out_set: HashSet::new(),
                },
            ]
        };

        // Actual `GenKillDecoratedImmutableBasicBlock`
        let actual_gen_kill_decorated_bb: LivenessDecoratedImmutableBasicBlock = immutable_bb.into();
        assert_eq!(expected_gen_kill_decorated_bb, actual_gen_kill_decorated_bb);
    }

    #[test]
    fn gen_kill_sets_for_bb() {
        let a = IdentI(data::Symbol::NonFunctionScopedSymbol(Rc::new(
            data::NonFunctionScopedSymbol::Int {
                name: "A".to_owned(),
            },
        )));
        let b = IdentI(data::Symbol::NonFunctionScopedSymbol(Rc::new(
            data::NonFunctionScopedSymbol::Int {
                name: "B".to_owned(),
            },
        )));
        let c = IdentI(data::Symbol::NonFunctionScopedSymbol(Rc::new(
            data::NonFunctionScopedSymbol::Int {
                name: "C".to_owned(),
            },
        )));
        let d = IdentI(data::Symbol::NonFunctionScopedSymbol(Rc::new(
            data::NonFunctionScopedSymbol::Int {
                name: "D".to_owned(),
            },
        )));

        let (t1, t2): (TempI, TempI) = (1.into(), 2.into());
        let bb_label: BBLabel = 0.into();

        // d = a + b * c
        let seq = vec![
            ThreeAddressCode::MulI {
                lhs: BinaryExprOperandI::LValue(LValueI::Id(b.clone())),
                rhs: BinaryExprOperandI::LValue(LValueI::Id(c.clone())),
                temp_result: t1,
            },
            ThreeAddressCode::AddI {
                lhs: BinaryExprOperandI::LValue(LValueI::Temp(t1)),
                rhs: BinaryExprOperandI::LValue(LValueI::Id(a.clone())),
                temp_result: t2,
            },
            ThreeAddressCode::StoreI {
                lhs: LValueI::Id(d.clone()),
                rhs: BinaryExprOperandI::LValue(LValueI::Temp(t2))
            }
        ];

        let immutable_bb: ImmutableBasicBlock = (bb_label, seq).into();

        // Expected `GenKillDecoratedImmutableBasicBlock`
        let expected_gen_kill_decorated_bb = LivenessDecoratedImmutableBasicBlock {
            label: immutable_bb.label(),
            seq: vec![
                LivenessDecoratedThreeAddressCode {
                    tac: ThreeAddressCode::MulI {
                        lhs: BinaryExprOperandI::LValue(LValueI::Id(b.clone())),
                        rhs: BinaryExprOperandI::LValue(LValueI::Id(c.clone())),
                        temp_result: t1,
                    },
                    gen_set: {
                        let mut gen = HashSet::new();
                        gen.insert(LValue::LValueI(LValueI::Id(b.clone())));
                        gen.insert(LValue::LValueI(LValueI::Id(c.clone())));
                        gen
                    },
                    kill_set: {
                        let mut kill = HashSet::new();
                        kill.insert(LValue::LValueI(LValueI::Temp(t1)));
                        kill
                    },
                    in_set: HashSet::new(),
                    out_set: HashSet::new(),
                },
                LivenessDecoratedThreeAddressCode {
                    tac: ThreeAddressCode::AddI {
                        lhs: BinaryExprOperandI::LValue(LValueI::Temp(t1)),
                        rhs: BinaryExprOperandI::LValue(LValueI::Id(a.clone())),
                        temp_result: t2,
                    },
                    gen_set: {
                        let mut gen = HashSet::new();
                        gen.insert(LValue::LValueI(LValueI::Temp(t1)));
                        gen.insert(LValue::LValueI(LValueI::Id(a.clone())));
                        gen
                    },
                    kill_set: {
                        let mut kill = HashSet::new();
                        kill.insert(LValue::LValueI(LValueI::Temp(t2)));
                        kill
                    },
                    in_set: HashSet::new(),
                    out_set: HashSet::new(),
                },
                LivenessDecoratedThreeAddressCode {
                    tac: ThreeAddressCode::StoreI {
                        lhs: LValueI::Id(d.clone()),
                        rhs: BinaryExprOperandI::LValue(LValueI::Temp(t2))
                    },
                    gen_set: {
                        let mut gen = HashSet::new();
                        gen.insert(LValue::LValueI(LValueI::Temp(t2)));
                        gen
                    },
                    kill_set: {
                        let mut kill = HashSet::new();
                        kill.insert(LValue::LValueI(LValueI::Id(d.clone())));
                        kill
                    },
                    in_set: HashSet::new(),
                    out_set: HashSet::new(),
                },
            ]
        };

        // Actual `GenKillDecoratedImmutableBasicBlock`
        let actual_gen_kill_decorated_bb: LivenessDecoratedImmutableBasicBlock = immutable_bb.into();
        assert_eq!(expected_gen_kill_decorated_bb, actual_gen_kill_decorated_bb);
    }
}