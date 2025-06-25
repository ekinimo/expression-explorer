use crate::ast::*;

use std::marker::PhantomData;
use std::ops::Index;

pub struct ChildIterator<'a, Pool, Id> {
    pool: &'a Pool,
    current_idx: usize,
    remaining: usize,
    is_empty: bool,
    _pd: PhantomData<Id>,
}

impl<'a, Pool, T> ChildIterator<'a, Pool, T>
where
    Pool: Children<T>,
    T: Copy + std::fmt::Debug + From<usize> + Into<usize>,
{
    pub fn new(pool: &'a Pool, self_id: T, arity: usize) -> Self {
        let self_idx: usize = self_id.into();
        Self {
            pool,
            current_idx: self_idx - 1,
            remaining: arity,
            _pd: PhantomData,
            is_empty: arity == 0,
        }
    }

    pub fn empty(pool: &'a Pool) -> Self {
        Self {
            pool,
            current_idx: 0,
            remaining: 0,
            _pd: PhantomData,
            is_empty: true,
        }
    }
}

impl<'a, Pool, T> Iterator for ChildIterator<'a, Pool, T>
where
    Pool: Children<T>,
    T: Copy + std::fmt::Debug + From<usize> + Into<usize>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty || self.remaining == 0 {
            return None;
        }

        let id = T::from(self.current_idx);
        let len = self.pool.length(id);

        if let Some(new_idx) = self.current_idx.checked_sub(len) {
            self.current_idx = new_idx;
        } else {
            self.remaining = 0;
            return Some(id);
        }

        self.remaining -= 1;
        Some(id)
    }
}

pub trait NodeInfo {
    fn arity(&self) -> usize;
    fn last(&self) -> Option<usize>;
}

impl NodeInfo for ExprNode {
    fn arity(&self) -> usize {
        match self {
            ExprNode::Number(_) | ExprNode::Variable(_) => 0,
            ExprNode::Call { arity, .. } | ExprNode::Struct { arity, .. } => *arity,
        }
    }

    fn last(&self) -> Option<usize> {
        match self {
            ExprNode::Number(_) | ExprNode::Variable(_) => None,
            ExprNode::Call { last, .. } | ExprNode::Struct { last, .. } => Some(*last),
        }
    }
}

impl NodeInfo for Action {
    fn arity(&self) -> usize {
        match self {
            Action::Number(_) | Action::Variable(_) => 0,
            Action::Call { arity, .. }
            | Action::Struct { arity, .. }
            | Action::VarCallName { arity, .. }
            | Action::VarStructName { arity, .. }
            | Action::Compute { arity, .. } => *arity,
        }
    }

    fn last(&self) -> Option<usize> {
        match self {
            Action::Number(_) | Action::Variable(_) => None,
            Action::Call { last, .. }
            | Action::Struct { last, .. }
            | Action::VarCallName { last, .. }
            | Action::VarStructName { last, .. }
            | Action::Compute { last, .. } => Some(*last),
        }
    }
}

impl NodeInfo for Pattern {
    fn arity(&self) -> usize {
        match self {
            Pattern::Number(_)
            | Pattern::Variable(_)
            | Pattern::AnyNumber(_)
            | Pattern::Wildcard(_) => 0,
            Pattern::Call { arity, .. }
            | Pattern::Struct { arity, .. }
            | Pattern::VarCallName { arity, .. }
            | Pattern::VarStructName { arity, .. } => *arity,
        }
    }

    fn last(&self) -> Option<usize> {
        match self {
            Pattern::Number(_)
            | Pattern::Variable(_)
            | Pattern::AnyNumber(_)
            | Pattern::Wildcard(_) => None,
            Pattern::Call { last, .. }
            | Pattern::Struct { last, .. }
            | Pattern::VarCallName { last, .. }
            | Pattern::VarStructName { last, .. } => Some(*last),
        }
    }
}

pub trait Children<Id>
where
    Self: Index<Id>,
    Id: Copy + From<usize> + Into<usize>,
{
    type Tree: NodeInfo;
    type ChildIterator<'a>: Iterator<Item = Id>
    where
        Self: 'a;

    fn get_node(&self, id: Id) -> &Self::Tree;

    fn children(&self, node_id: Id) -> Self::ChildIterator<'_>;

    fn length(&self, node_id: Id) -> usize {
        let node = self.get_node(node_id);
        match node.last() {
            Some(last) if node.arity() > 0 => {
                let node_idx: usize = node_id.into();
                let first_child_pos = node_idx - last;
                (first_child_pos..=node_idx).count()
            }
            _ => 1,
        }
    }

    fn total_len(&self, node_id: Id) -> usize {
        let node = self.get_node(node_id);
        match node.last() {
            Some(last) if node.arity() > 0 => {
                let len = self.length(node_id);
                let first_child_id = Id::from(node_id.into() - last);
                len + self.total_len(first_child_id)
            }
            _ => 1,
        }
    }

    fn get_child_slice(&self, node_id: Id) -> &[Self::Tree]
    where
        Self: Index<std::ops::Range<Id>, Output = [Self::Tree]>,
    {
        let node = self.get_node(node_id);
        let node_idx: usize = node_id.into();
        match node.last() {
            Some(last) if node.arity() > 0 => {
                let first_child_pos = node_idx - last;
                let start = Id::from(first_child_pos);
                let end = Id::from(node_idx + 1);
                &self[start..end]
            }
            _ => {
                let start = node_id;
                let end = Id::from(node_idx + 1);
                &self[start..end]
            }
        }
    }

    fn get_full_slice(&self, node_id: Id) -> &[Self::Tree]
    where
        Self: Index<std::ops::Range<Id>, Output = [Self::Tree]>,
    {
        let len = self.length(node_id);
        let node_idx: usize = node_id.into();

        let start_idx = (node_idx + 1).saturating_sub(len);

        let start = Id::from(start_idx);
        let end = Id::from(node_idx + 1);
        &self[start..end]
    }

    fn get(&self, node_id: Id) -> Option<&<Self as Index<Id>>::Output> {
        Some(&self[node_id])
    }

    fn child(&self, node_id: Id, index: usize) -> Option<Id> {
        self.children(node_id).nth(index)
    }
}

pub trait LeafNode<Id: From<usize> + Into<usize> + Copy> {
    type Tree;
    fn get_storage(&self) -> &[Self::Tree];
}

impl NodeInfo for Function {
    fn arity(&self) -> usize {
        0
    }
    fn last(&self) -> Option<usize> {
        None
    }
}

impl NodeInfo for String {
    fn arity(&self) -> usize {
        0
    }
    fn last(&self) -> Option<usize> {
        None
    }
}

impl NodeInfo for Rule {
    fn arity(&self) -> usize {
        0
    }
    fn last(&self) -> Option<usize> {
        None
    }
}

impl<Id> Children<Id> for crate::Pool
where
    Id: Copy + From<usize> + Into<usize> + std::fmt::Debug,
    crate::Pool:
        Index<Id> + Index<std::ops::Range<Id>, Output = [<crate::Pool as Index<Id>>::Output]>,
    <crate::Pool as Index<Id>>::Output: NodeInfo,
    <crate::Pool as Index<Id>>::Output: Sized,
{
    type Tree = <Self as Index<Id>>::Output;
    type ChildIterator<'a> = ChildIterator<'a, crate::Pool, Id>;

    fn get_node(&self, id: Id) -> &Self::Tree {
        &self[id]
    }

    fn children(&self, node_id: Id) -> Self::ChildIterator<'_> {
        let node = self.get_node(node_id);
        let arity = node.arity();
        if arity > 0 {
            ChildIterator::new(self, node_id, arity)
        } else {
            ChildIterator::empty(self)
        }
    }
}
