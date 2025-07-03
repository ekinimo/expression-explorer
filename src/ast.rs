use crate::idx::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum ExprNode {
    Number(i32),
    Variable(NameId),
    Call {
        fun: FunctionId,
        last: usize,
        arity: usize,
    },
    Struct {
        name: NameId,
        last: usize,
        arity: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Function {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Negate,
    Plus,
    Custom(NameId),
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub enum Pattern {
    Number(i32),
    Variable(NameId),
    AnyNumber(NameId),
    Wildcard(NameId),
    Call {
        fun: FunctionId,
        last: usize,
        arity: usize,
    },
    Struct {
        name: NameId,
        last: usize,
        arity: usize,
    },
    VarCallName {
        var: NameId,
        last: usize,
        arity: usize,
    },
    VarStructName {
        var: NameId,
        last: usize,
        arity: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub enum Action {
    Number(i32),
    Variable(NameId),
    Call {
        fun: FunctionId,
        last: usize,
        arity: usize,
    },
    Struct {
        name: NameId,
        last: usize,
        arity: usize,
    },
    VarCallName {
        var: NameId,
        last: usize,
        arity: usize,
    },
    VarStructName {
        var: NameId,
        last: usize,
        arity: usize,
    },

    Compute {
        op: ComputeOp,
        last: usize,
        arity: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub enum ComputeOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub struct Rule {
    pub name: NameId,
    pub pattern: PatternId,
    pub action: ActionId,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub struct Ruleset {
    pub name: NameId,
    pub rules_start: usize,
    pub rules_end: usize,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Function::Add => write!(f, "+"),
            Function::Subtract => write!(f, "-"),
            Function::Multiply => write!(f, "*"),
            Function::Divide => write!(f, "/"),
            Function::Power => write!(f, "^"),
            Function::Negate => write!(f, "neg"),
            Function::Plus => write!(f, "+"),
            Function::Custom(name_id) => write!(f, "custom_{}", name_id.0),
        }
    }
}
