#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum NumType {
    Int,
    Float,
}

pub mod data {
    use crate::symbol_table::symbol::NumType;
    use std::rc::Rc;

    #[derive(Debug, Eq, PartialEq, Copy, Clone)]
    pub enum DataType {
        String,
        Num(NumType),
    }

    /// Represents a symbol declared in the program.
    /// Symbol maybe a `DataSymbol` - declared in
    /// global or anonymous scopes, ot it might be a
    /// `FunctionDataSymbol` - declared in function scopes.
    #[derive(Debug, Clone, Hash, Eq, PartialEq, derive_more::Display)]
    pub enum Symbol {
        NonFunctionScopedSymbol(Rc<NonFunctionScopedSymbol>),
        FunctionScopedSymbol(Rc<FunctionScopedSymbol>),
    }

    /// Represents a symbol declared in the global
    /// scope or an anonymous scope (if blocks, for loops etc.),
    /// in the program to represent data - string, int or a float.
    #[derive(Debug, PartialEq, Clone, Hash, Eq, derive_more::Display)]
    pub enum NonFunctionScopedSymbol {
        #[display(fmt = "name {} type STRING value {}\n", name, value)]
        String { name: String, value: String },
        #[display(fmt = "name {} type INT\n", name)]
        Int { name: String },
        #[display(fmt = "name {} type FLOAT\n", name)]
        Float { name: String },
    }

    impl NonFunctionScopedSymbol {
        pub fn name(&self) -> &str {
            match self {
                NonFunctionScopedSymbol::String { name, value } => name,
                NonFunctionScopedSymbol::Int { name } => name,
                NonFunctionScopedSymbol::Float { name } => name,
            }
        }
    }

    /// Represents the type of the function
    /// symbol - parameter or local
    #[derive(Debug, Eq, Clone, PartialEq, Hash, derive_more::Display)]
    pub enum FunctionScopedSymbolType {
        #[display(fmt = "P")]
        Parameter,
        #[display(fmt = "L")]
        Local,
    }

    /// Represents a symbol in the scope of a
    /// function. The symbol is either a function
    /// parameter or a local variable and can be
    /// an int or a float.
    #[derive(Debug, PartialEq, Clone, Hash, Eq, derive_more::Display)]
    pub enum FunctionScopedSymbol {
        #[display(fmt = "name: {}{} type INT\n", symbol_type, index)]
        Int {
            symbol_type: FunctionScopedSymbolType,
            index: u32,
        },
        #[display(fmt = "name: {}{} type FLOAT\n", symbol_type, index)]
        Float {
            symbol_type: FunctionScopedSymbolType,
            index: u32,
        },
    }
}

pub mod function {
    use crate::symbol_table::symbol::NumType;

    /// Represents possible return types
    /// in a function.
    #[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
    pub enum ReturnType {
        Num(NumType),
        Void,
    }

    /// Represents function or non-data
    /// symbols in the program.
    #[derive(Debug, PartialEq, Clone, Hash, Eq)]
    pub struct Symbol {
        name: String,
        return_type: ReturnType,
        params_list: Vec<NumType>,
        locals_list: Vec<NumType>,
    }

    impl Symbol {
        pub fn new(name: String, return_type: ReturnType, param_list: Vec<NumType>, locals_list: Vec<NumType>) -> Self {
            Self {
                name,
                return_type,
                params_list: param_list,
                locals_list,
            }
        }

        pub fn name(&self) -> &str {
            &self.name
        }
    }
}
