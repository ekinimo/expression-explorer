use crate::children::Children;

pub mod ast;
pub mod children;
pub mod display;
pub mod graph;
pub mod idx;
pub mod pool;

pub use ast::*;
pub use display::*;
pub use idx::*;
pub use pool::*;

pub mod parser;
pub mod rules;
pub mod search;

pub mod ui;

pub use parser::action_parser::ActionParser;
pub use parser::compute_parser::ComputeParser;
pub use parser::expr_parser::ExprParser;
pub use parser::pattern_parser::PatternParser;
pub use parser::ruleset_parser::RulesetParser;

impl Pool {
    pub fn expr_eq(&self, node1_id: ExprId, node2_id: ExprId) -> bool {
        let mut stack = vec![(node1_id, node2_id)];

        loop {
            let Some((node1_id, node2_id)) = stack.pop() else {
                return true;
            };

            let Some(node1) = self.get(node1_id) else {
                return false;
            };
            let Some(node2) = self.get(node2_id) else {
                return false;
            };

            match (node1, node2) {
                (ExprNode::Number(i), ExprNode::Number(j)) if i == j => continue,
                (ExprNode::Variable(i), ExprNode::Variable(j)) if i == j => continue,
                (
                    ExprNode::Call {
                        fun: fun1,
                        arity: len1,
                        ..
                    },
                    ExprNode::Call {
                        fun: fun2,
                        arity: len2,
                        ..
                    },
                ) => {
                    if fun1 != fun2 || len1 != len2 {
                        return false;
                    }
                    let children1: Vec<_> = self.children(node1_id).collect();
                    let children2: Vec<_> = self.children(node2_id).collect();

                    for (child1, child2) in children1.into_iter().zip(children2.into_iter()) {
                        stack.push((child1, child2));
                    }
                }
                _ => return false,
            }
        }
    }
}
