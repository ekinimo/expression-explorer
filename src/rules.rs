use crate::children::Children;
use crate::{
    Action, ActionId, ComputeOp, ExprId, ExprNode, FunctionId, NameId, Pattern, PatternId, Pool,
    RuleId,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CapturedValue {
    Expression(ExprId),
    Function(FunctionId),
    StructName(NameId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub root: ExprId,
    pub offset: ExprId,

    pub rule_id: RuleId,
    pub captures: HashMap<NameId, CapturedValue>,
}

impl Pool {
    pub fn find_matches(&self, root: ExprId) -> Vec<Match> {
        let mut matches = Vec::new();
        self.find_matches_recursive(root, &mut matches);
        matches
    }

    fn find_matches_recursive(&self, node_id: ExprId, matches: &mut Vec<Match>) {
        let root = node_id;
        let mut stack = vec![(node_id)];

        while let Some(current_node_id) = stack.pop() {
            if let Some(node) = self.get(current_node_id) {
                for (rule_id, rule) in self.rules.iter().enumerate() {
                    let mut captures = HashMap::new();
                    if self.pattern_matches(rule.pattern, current_node_id, &mut captures) {
                        matches.push(Match {
                            root,
                            offset: current_node_id,
                            rule_id: RuleId::new(rule_id),
                            captures,
                        });
                    }
                }

                match node {
                    ExprNode::Call { .. } | ExprNode::Struct { .. } => {
                        for child in self.children(current_node_id) {
                            stack.push(child);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn pattern_matches(
        &self,
        pattern_id: PatternId,
        node_id: ExprId,
        captures: &mut HashMap<NameId, CapturedValue>,
    ) -> bool {
        let (pattern, node) = (self[pattern_id], self[node_id]);
        match (pattern, node) {
            (Pattern::Number(p_num), ExprNode::Number(n_num)) => p_num == n_num,
            (Pattern::Variable(p_var_id), ExprNode::Variable(n_var_id)) => p_var_id == n_var_id,

            (Pattern::AnyNumber(capture_id), ExprNode::Number(_))
                if !captures.contains_key(&capture_id) =>
            {
                captures.insert(capture_id, CapturedValue::Expression(node_id));
                true
            }
            (Pattern::Wildcard(wildcard_id), _) if !captures.contains_key(&wildcard_id) => {
                captures.insert(wildcard_id, CapturedValue::Expression(node_id));
                true
            }
            (Pattern::AnyNumber(wildcard_id), ExprNode::Number(_))
            | (Pattern::Wildcard(wildcard_id), _) => {
                if let Some(CapturedValue::Expression(registered_id)) = captures.get(&wildcard_id) {
                    self.expr_eq(*registered_id, node_id)
                } else {
                    false
                }
            }

            (
                Pattern::Call {
                    fun: p_fun,
                    arity: p_len,
                    ..
                },
                ExprNode::Call {
                    fun: n_fun,
                    arity: n_len,
                    ..
                },
            ) => {
                if p_fun != n_fun || p_len != n_len {
                    return false;
                }
                let pattern_children = self.children(pattern_id);
                let node_children = self.children(node_id);

                pattern_children
                    .into_iter()
                    .zip(node_children)
                    .all(|(p_child, n_child)| self.pattern_matches(p_child, n_child, captures))
            }

            (
                Pattern::Struct {
                    name: p_fun,
                    arity: p_len,
                    ..
                },
                ExprNode::Struct {
                    name: n_fun,
                    arity: n_len,
                    ..
                },
            ) => {
                if p_fun != n_fun || p_len != n_len {
                    return false;
                }
                let pattern_children = self.children(pattern_id);
                let node_children = self.children(node_id);

                pattern_children
                    .into_iter()
                    .zip(node_children)
                    .all(|(p_child, n_child)| self.pattern_matches(p_child, n_child, captures))
            }

            (
                Pattern::VarCallName {
                    var: p_var,
                    arity: p_len,
                    ..
                },
                ExprNode::Call {
                    fun: n_fun,
                    arity: n_len,
                    ..
                },
            ) => {
                if p_len != n_len {
                    return false;
                }
                if let std::collections::hash_map::Entry::Vacant(e) = captures.entry(p_var) {
                    e.insert(CapturedValue::Function(n_fun));
                } else if let Some(CapturedValue::Function(captured_fun)) = captures.get(&p_var) {
                    if captured_fun != &n_fun {
                        return false;
                    }
                } else {
                    return false;
                }

                let pattern_children = self.children(pattern_id);
                let node_children = self.children(node_id);

                pattern_children
                    .into_iter()
                    .zip(node_children)
                    .all(|(p_child, n_child)| self.pattern_matches(p_child, n_child, captures))
            }

            (
                Pattern::VarStructName {
                    var: p_var,
                    arity: p_len,
                    ..
                },
                ExprNode::Struct {
                    name: n_name,
                    arity: n_len,
                    ..
                },
            ) => {
                if p_len != n_len {
                    return false;
                }
                if let std::collections::hash_map::Entry::Vacant(e) = captures.entry(p_var) {
                    e.insert(CapturedValue::StructName(n_name));
                } else if let Some(CapturedValue::StructName(captured_name)) = captures.get(&p_var)
                {
                    if captured_name != &n_name {
                        return false;
                    }
                } else {
                    return false;
                }

                let pattern_children = self.children(pattern_id);
                let node_children = self.children(node_id);

                pattern_children
                    .into_iter()
                    .zip(node_children)
                    .all(|(p_child, n_child)| self.pattern_matches(p_child, n_child, captures))
            }

            _ => false,
        }
    }

    pub fn apply_rule(&mut self, match_: &Match) -> Option<ExprId> {
        let rule = self[match_.rule_id];

        let mut replacement_vec = Vec::new();
        self.build_action_simple(rule.action, &match_.captures, &mut replacement_vec);

        if replacement_vec.is_empty() {
            return None;
        }

        if match_.root == match_.offset {
            for (node, prov) in replacement_vec.drain(..) {
                self.exprs.push(node);
                self.locations.push(prov);
            }
            let new_root = ExprId(self.exprs.len() - 1);
            self.mark_expr_end(new_root);
            self.add_transformation(match_.root, new_root, match_.rule_id);
            return Some(new_root);
        }

        let mut root_vec: Vec<(ExprNode, crate::pool::Provenance)> = Vec::new();
        self.copy_expression_to_vec(match_.root, &mut root_vec);

        let relative_pos = match_.root.0 - match_.offset.0;
        let target_slice_len = self.get_full_slice(match_.offset).len();
        let root_slice_len = self.get_full_slice(match_.root).len();

        let target_end = root_slice_len.saturating_sub(relative_pos);
        let target_start = target_end.saturating_sub(target_slice_len);

        let replacement_nodes: Vec<_> = replacement_vec.into_iter().collect();
        let replacement_len = replacement_nodes.len();

        root_vec.splice(target_start..target_end, replacement_nodes);

        let size_delta = replacement_len as i32 - target_slice_len as i32;
        if size_delta != 0 {
            self.fix_indices_after_splice(&mut root_vec, target_start, size_delta);
        }

        for (node, prov) in root_vec.drain(..) {
            self.exprs.push(node);
            self.locations.push(prov);
        }
        let new_root = ExprId(self.exprs.len() - 1);
        self.mark_expr_end(new_root);

        self.add_transformation(match_.root, new_root, match_.rule_id);

        Some(new_root)
    }

    fn fix_indices_after_splice(
        &self,
        expr_vec: &mut [(ExprNode, crate::pool::Provenance)],
        splice_start: usize,
        size_delta: i32,
    ) {
        for (i, (node, _)) in expr_vec.iter_mut().enumerate() {
            match node {
                ExprNode::Call { last, .. } | ExprNode::Struct { last, .. } => {
                    if *last <= i {
                        let first_child_pos = i - *last;
                        if first_child_pos < splice_start && i >= splice_start {
                            if size_delta > 0 {
                                *last += size_delta as usize;
                            } else {
                                *last = last.saturating_sub((-size_delta) as usize);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn build_action_simple(
        &mut self,
        action_id: ActionId,
        captures: &HashMap<NameId, CapturedValue>,
        output: &mut Vec<(ExprNode, crate::pool::Provenance)>,
    ) {
        let action = self[action_id];

        let provenance = crate::pool::Provenance::Rule {
            rule_id: RuleId::new(0),
            source_node: ExprId::new(0),
            source_location: crate::pool::Location::new(0, 0),
        };

        match action {
            Action::Number(n) => {
                output.push((ExprNode::Number(n), provenance));
            }

            Action::Variable(var_id) => {
                if let Some(CapturedValue::Expression(expr_id)) = captures.get(&var_id) {
                    self.copy_expression_to_vec(*expr_id, output);
                } else {
                    output.push((ExprNode::Variable(var_id), provenance));
                }
            }

            Action::Call { fun, arity, .. } => {
                let start_pos = output.len();
                let children: Vec<_> = self.children(action_id).collect();
                for child_id in children.into_iter().rev() {
                    self.build_action_simple(child_id, captures, output);
                }
                let last = if arity > 0 {
                    output.len() - start_pos
                } else {
                    0
                };
                output.push((ExprNode::Call { fun, last, arity }, provenance));
            }

            Action::Struct { name, arity, .. } => {
                let start_pos = output.len();
                let children: Vec<_> = self.children(action_id).collect();
                for child_id in children.into_iter().rev() {
                    self.build_action_simple(child_id, captures, output);
                }
                let last = if arity > 0 {
                    output.len() - start_pos
                } else {
                    0
                };
                output.push((ExprNode::Struct { name, last, arity }, provenance));
            }

            Action::VarCallName { var, arity, .. } => {
                if let Some(CapturedValue::Function(fun)) = captures.get(&var) {
                    let start_pos = output.len();
                    let children: Vec<_> = self.children(action_id).collect();
                    for child_id in children.into_iter().rev() {
                        self.build_action_simple(child_id, captures, output);
                    }
                    let last = if arity > 0 {
                        output.len() - start_pos
                    } else {
                        0
                    };
                    output.push((
                        ExprNode::Call {
                            fun: *fun,
                            last,
                            arity,
                        },
                        provenance,
                    ));
                }
            }

            Action::VarStructName { var, arity, .. } => {
                if let Some(CapturedValue::StructName(name)) = captures.get(&var) {
                    let start_pos = output.len();
                    let children: Vec<_> = self.children(action_id).collect();
                    for child_id in children.into_iter().rev() {
                        self.build_action_simple(child_id, captures, output);
                    }
                    let last = if arity > 0 {
                        output.len() - start_pos
                    } else {
                        0
                    };
                    output.push((
                        ExprNode::Struct {
                            name: *name,
                            last,
                            arity,
                        },
                        provenance,
                    ));
                }
            }

            Action::Compute { op, arity, .. } => {
                let children: Vec<_> = self.children(action_id).collect();
                if children.len() == arity {
                    let mut args = Vec::new();
                    for child_id in children {
                        let child_action = self[child_id];
                        match child_action {
                            Action::Number(n) => {
                                args.push(n);
                            }
                            Action::Variable(var_id) => {
                                if let Some(CapturedValue::Expression(expr_id)) =
                                    captures.get(&var_id)
                                {
                                    if let Some(value) =
                                        self.evaluate_numeric_expr(*expr_id, captures)
                                    {
                                        args.push(value);
                                    } else {
                                        return;
                                    }
                                } else {
                                    return;
                                }
                            }
                            _ => return,
                        }
                    }
                    if let Some(result) = self.compute_operation_simple(op, &args) {
                        output.push((ExprNode::Number(result), provenance));
                    }
                }
            }
        }
    }

    fn copy_expression_to_vec(
        &self,
        expr_id: ExprId,
        output: &mut Vec<(ExprNode, crate::pool::Provenance)>,
    ) {
        let expr_slice = self.get_full_slice(expr_id);
        for &node in expr_slice {
            let provenance = self.get_provenance(expr_id).cloned().unwrap_or_else(|| {
                crate::pool::Provenance::Parsed(crate::pool::Location::new(0, 0))
            });
            output.push((node, provenance));
        }
    }

    fn compute_operation_simple(&self, op: ComputeOp, args: &[i32]) -> Option<i32> {
        match op {
            ComputeOp::Add => Some(args.iter().sum()),
            ComputeOp::Subtract if args.len() == 2 => Some(args[0] - args[1]),
            ComputeOp::Multiply => Some(args.iter().product()),
            ComputeOp::Divide if args.len() == 2 && args[1] != 0 => Some(args[0] / args[1]),
            ComputeOp::Power if args.len() == 2 && args[1] >= 0 => {
                Some(args[0].pow(args[1] as u32))
            }
            ComputeOp::Negate if args.len() == 1 => Some(-args[0]),
            _ => None,
        }
    }

    fn evaluate_numeric_expr(
        &self,
        expr_id: ExprId,
        captures: &HashMap<NameId, CapturedValue>,
    ) -> Option<i32> {
        match self.get(expr_id)? {
            ExprNode::Number(n) => Some(*n),
            ExprNode::Variable(var_id) => {
                if let Some(CapturedValue::Expression(captured_expr)) = captures.get(var_id) {
                    self.evaluate_numeric_expr(*captured_expr, captures)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
