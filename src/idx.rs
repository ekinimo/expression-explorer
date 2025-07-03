use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExprId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NameId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RulesetId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EquivalenceGroupId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatternId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuleId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActionId(pub usize);

impl ExprId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

impl NameId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

impl RulesetId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

impl FunctionId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

impl EquivalenceGroupId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

impl PatternId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

impl RuleId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

impl ActionId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn get(&self) -> usize {
        self.0
    }
}

impl fmt::Display for NameId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for ExprId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for ExprId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<ExprId> for usize {
    fn from(id: ExprId) -> Self {
        id.0
    }
}

impl From<usize> for NameId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<NameId> for usize {
    fn from(id: NameId) -> Self {
        id.0
    }
}

impl From<usize> for RulesetId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<RulesetId> for usize {
    fn from(id: RulesetId) -> Self {
        id.0
    }
}

impl From<usize> for FunctionId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<FunctionId> for usize {
    fn from(id: FunctionId) -> Self {
        id.0
    }
}

impl From<usize> for EquivalenceGroupId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<EquivalenceGroupId> for usize {
    fn from(id: EquivalenceGroupId) -> Self {
        id.0
    }
}

impl From<usize> for PatternId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<PatternId> for usize {
    fn from(id: PatternId) -> Self {
        id.0
    }
}

impl From<usize> for RuleId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<RuleId> for usize {
    fn from(id: RuleId) -> Self {
        id.0
    }
}

impl From<usize> for ActionId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<ActionId> for usize {
    fn from(id: ActionId) -> Self {
        id.0
    }
}
