use crate::ast::*;
use crate::children::Children;
use crate::idx::*;
use std::ops::Index;

pub trait DisplayNode<Id>
where
    Self: Index<Id> + Children<Id>,
    Id: Copy + From<usize> + Into<usize>,
{
    fn display_with_children(&self, node_id: Id) -> String;
}

impl DisplayNode<FunctionId> for crate::Pool {
    fn display_with_children(&self, fun_id: FunctionId) -> String {
        self.display_function(fun_id)
    }
}

impl DisplayNode<NameId> for crate::Pool {
    fn display_with_children(&self, name_id: NameId) -> String {
        self.display_name(name_id)
    }
}

impl DisplayNode<ExprId> for crate::Pool {
    fn display_with_children(&self, node_id: ExprId) -> String {
        enum StackItem {
            Process(ExprId),
            Combine(ExprId, Vec<String>),
        }

        let mut stack = vec![StackItem::Process(node_id)];
        let mut results = Vec::new();

        while let Some(item) = stack.pop() {
            match item {
                StackItem::Process(id) => {
                    let node = self[id];
                    match node {
                        ExprNode::Number(n) => {
                            results.push(n.to_string());
                        }
                        ExprNode::Variable(name_id) => {
                            results.push(self.display_name(name_id));
                        }
                        ExprNode::Call { .. } | ExprNode::Struct { .. } => {
                            let children: Vec<_> = self.children(id).collect();
                            if children.is_empty() {
                                match node {
                                    ExprNode::Call { fun, .. } => {
                                        results.push(format!("{}()", self.display_function(fun)));
                                    }
                                    ExprNode::Struct { name, .. } => {
                                        results.push(format!("{}{{ }}", self.display_name(name)));
                                    }
                                    _ => unreachable!(),
                                }
                            } else {
                                stack.push(StackItem::Combine(id, Vec::new()));
                                for child in children.into_iter().rev() {
                                    stack.push(StackItem::Process(child));
                                }
                            }
                        }
                    }
                }
                StackItem::Combine(id, mut child_results) => {
                    let node = self[id];
                    let arity = match node {
                        ExprNode::Call { arity, .. } | ExprNode::Struct { arity, .. } => arity,
                        _ => 0,
                    };

                    for _ in 0..arity {
                        if let Some(child_result) = results.pop() {
                            child_results.push(child_result);
                        }
                    }

                    let result = match node {
                        ExprNode::Call { fun, arity, .. } => {
                            match (self.get(fun), arity, child_results.len()) {
                                (Some(Function::Add), 2, 2) => {
                                    format!("({} + {})", child_results[0], child_results[1])
                                }
                                (Some(Function::Subtract), 2, 2) => {
                                    format!("({} - {})", child_results[0], child_results[1])
                                }
                                (Some(Function::Multiply), 2, 2) => {
                                    format!("({} * {})", child_results[0], child_results[1])
                                }
                                (Some(Function::Divide), 2, 2) => {
                                    format!("({} / {})", child_results[0], child_results[1])
                                }
                                (Some(Function::Power), 2, 2) => {
                                    format!("({} ^ {})", child_results[0], child_results[1])
                                }
                                (Some(Function::Negate), 1, 1) => {
                                    format!("(-{})", child_results[0])
                                }
                                (Some(Function::Plus), 1, 1) => {
                                    format!("(+{})", child_results[0])
                                }
                                _ => {
                                    format!(
                                        "{}({})",
                                        self.display_function(fun),
                                        child_results.join(", ")
                                    )
                                }
                            }
                        }
                        ExprNode::Struct { name, .. } => {
                            format!(
                                "{}{{ {} }}",
                                self.display_name(name),
                                child_results.join(", ")
                            )
                        }
                        _ => unreachable!(),
                    };

                    results.push(result);
                }
            }
        }

        results.pop().unwrap_or_else(|| "<error>".to_string())
    }
}

impl DisplayNode<PatternId> for crate::Pool {
    fn display_with_children(&self, node_id: PatternId) -> String {
        enum StackItem {
            Process(PatternId),
            Combine(PatternId, Vec<String>),
        }

        let mut stack = vec![StackItem::Process(node_id)];
        let mut results = Vec::new();

        while let Some(item) = stack.pop() {
            match item {
                StackItem::Process(id) => {
                    let node = self[id];
                    match node {
                        Pattern::Number(n) => {
                            results.push(n.to_string());
                        }
                        Pattern::Variable(name_id) => {
                            results.push(self.display_name(name_id));
                        }
                        Pattern::AnyNumber(name_id) => {
                            results.push(format!("#{}", self.display_name(name_id)));
                        }
                        Pattern::Wildcard(name_id) => {
                            results.push(format!("?{}", self.display_name(name_id)));
                        }
                        Pattern::Call { .. }
                        | Pattern::Struct { .. }
                        | Pattern::VarCallName { .. }
                        | Pattern::VarStructName { .. } => {
                            let children: Vec<_> = self.children(id).collect();
                            if children.is_empty() {
                                match node {
                                    Pattern::Call { fun, .. } => {
                                        results.push(format!("{}()", self.display_function(fun)));
                                    }
                                    Pattern::Struct { name, .. } => {
                                        results.push(format!("{}{{ }}", self.display_name(name)));
                                    }
                                    Pattern::VarCallName { var, .. } => {
                                        results.push(format!("?{}()", self.display_name(var)));
                                    }
                                    Pattern::VarStructName { var, .. } => {
                                        results.push(format!("?{}{{ }}", self.display_name(var)));
                                    }
                                    _ => unreachable!(),
                                }
                            } else {
                                stack.push(StackItem::Combine(id, Vec::new()));
                                for child in children.into_iter().rev() {
                                    stack.push(StackItem::Process(child));
                                }
                            }
                        }
                    }
                }
                StackItem::Combine(id, mut child_results) => {
                    let node = self[id];
                    let arity = match node {
                        Pattern::Call { arity, .. }
                        | Pattern::Struct { arity, .. }
                        | Pattern::VarCallName { arity, .. }
                        | Pattern::VarStructName { arity, .. } => arity,
                        _ => 0,
                    };

                    for _ in 0..arity {
                        if let Some(child_result) = results.pop() {
                            child_results.push(child_result);
                        }
                    }

                    let result = match node {
                        Pattern::Call { fun, arity, .. } => match (self.get(fun), arity) {
                            (Some(Function::Add), 2) => {
                                format!("({} + {})", child_results[0], child_results[1])
                            }
                            (Some(Function::Subtract), 2) => {
                                format!("({} - {})", child_results[0], child_results[1])
                            }
                            (Some(Function::Multiply), 2) => {
                                format!("({} * {})", child_results[0], child_results[1])
                            }
                            (Some(Function::Divide), 2) => {
                                format!("({} / {})", child_results[0], child_results[1])
                            }
                            (Some(Function::Power), 2) => {
                                format!("({} ^ {})", child_results[0], child_results[1])
                            }
                            (Some(Function::Negate), 1) => {
                                format!("(-{})", child_results[0])
                            }
                            (Some(Function::Plus), 1) => {
                                format!("(+{})", child_results[0])
                            }
                            _ => {
                                format!(
                                    "{}({})",
                                    self.display_function(fun),
                                    child_results.join(", ")
                                )
                            }
                        },
                        Pattern::Struct { name, .. } => {
                            format!(
                                "{}{{ {} }}",
                                self.display_name(name),
                                child_results.join(", ")
                            )
                        }
                        Pattern::VarCallName { var, .. } => {
                            format!("?{}({})", self.display_name(var), child_results.join(", "))
                        }
                        Pattern::VarStructName { var, .. } => {
                            format!(
                                "?{}{{ {} }}",
                                self.display_name(var),
                                child_results.join(", ")
                            )
                        }
                        _ => unreachable!(),
                    };

                    results.push(result);
                }
            }
        }

        results.pop().unwrap_or_else(|| "<error>".to_string())
    }
}

impl DisplayNode<ActionId> for crate::Pool {
    fn display_with_children(&self, node_id: ActionId) -> String {
        enum StackItem {
            Process(ActionId),
            Combine(ActionId, Vec<String>),
        }

        let mut stack = vec![StackItem::Process(node_id)];
        let mut results = Vec::new();

        while let Some(item) = stack.pop() {
            match item {
                StackItem::Process(id) => {
                    let node = self[id];
                    match node {
                        Action::Number(n) => {
                            results.push(n.to_string());
                        }
                        Action::Variable(name_id) => {
                            results.push(self.display_name(name_id));
                        }
                        Action::Call { .. }
                        | Action::Struct { .. }
                        | Action::VarCallName { .. }
                        | Action::VarStructName { .. }
                        | Action::Compute { .. } => {
                            let children: Vec<_> = self.children(id).collect();
                            if children.is_empty() {
                                match node {
                                    Action::Call { fun, .. } => {
                                        results.push(format!("{}()", self.display_function(fun)));
                                    }
                                    Action::Struct { name, .. } => {
                                        results.push(format!("{}{{ }}", self.display_name(name)));
                                    }
                                    Action::VarCallName { var, .. } => {
                                        results.push(format!("?{}()", self.display_name(var)));
                                    }
                                    Action::VarStructName { var, .. } => {
                                        results.push(format!("?{}{{ }}", self.display_name(var)));
                                    }
                                    Action::Compute { op, .. } => {
                                        let op_str = match op {
                                            ComputeOp::Add => "+",
                                            ComputeOp::Subtract => "-",
                                            ComputeOp::Multiply => "*",
                                            ComputeOp::Divide => "/",
                                            ComputeOp::Power => "^",
                                            ComputeOp::Negate => "neg",
                                        };
                                        results.push(format!("[{}()]", op_str));
                                    }
                                    _ => unreachable!(),
                                }
                            } else {
                                stack.push(StackItem::Combine(id, Vec::new()));
                                for child in children.into_iter().rev() {
                                    stack.push(StackItem::Process(child));
                                }
                            }
                        }
                    }
                }
                StackItem::Combine(id, mut child_results) => {
                    let node = self[id];
                    let arity = match node {
                        Action::Call { arity, .. }
                        | Action::Struct { arity, .. }
                        | Action::VarCallName { arity, .. }
                        | Action::VarStructName { arity, .. }
                        | Action::Compute { arity, .. } => arity,
                        _ => 0,
                    };

                    for _ in 0..arity {
                        if let Some(child_result) = results.pop() {
                            child_results.push(child_result);
                        }
                    }

                    let result = match node {
                        Action::Call { fun, arity, .. } => match (self.get(fun), arity) {
                            (Some(Function::Add), 2) => {
                                format!("({} + {})", child_results[0], child_results[1])
                            }
                            (Some(Function::Subtract), 2) => {
                                format!("({} - {})", child_results[0], child_results[1])
                            }
                            (Some(Function::Multiply), 2) => {
                                format!("({} * {})", child_results[0], child_results[1])
                            }
                            (Some(Function::Divide), 2) => {
                                format!("({} / {})", child_results[0], child_results[1])
                            }
                            (Some(Function::Power), 2) => {
                                format!("({} ^ {})", child_results[0], child_results[1])
                            }
                            (Some(Function::Negate), 1) => {
                                format!("(-{})", child_results[0])
                            }
                            (Some(Function::Plus), 1) => {
                                format!("(+{})", child_results[0])
                            }
                            _ => {
                                format!(
                                    "{}({})",
                                    self.display_function(fun),
                                    child_results.join(", ")
                                )
                            }
                        },
                        Action::Struct { name, .. } => {
                            format!(
                                "{}{{ {} }}",
                                self.display_name(name),
                                child_results.join(", ")
                            )
                        }
                        Action::VarCallName { var, .. } => {
                            format!("?{}({})", self.display_name(var), child_results.join(", "))
                        }
                        Action::VarStructName { var, .. } => {
                            format!(
                                "?{}{{ {} }}",
                                self.display_name(var),
                                child_results.join(", ")
                            )
                        }
                        Action::Compute { op, arity, .. } => match (op, arity) {
                            (ComputeOp::Add, 2) => {
                                format!("[{} + {}]", child_results[0], child_results[1])
                            }
                            (ComputeOp::Subtract, 2) => {
                                format!("[{} - {}]", child_results[0], child_results[1])
                            }
                            (ComputeOp::Multiply, 2) => {
                                format!("[{} * {}]", child_results[0], child_results[1])
                            }
                            (ComputeOp::Divide, 2) => {
                                format!("[{} / {}]", child_results[0], child_results[1])
                            }
                            (ComputeOp::Power, 2) => {
                                format!("[{} ^ {}]", child_results[0], child_results[1])
                            }
                            (ComputeOp::Negate, 1) => {
                                format!("[-{}]", child_results[0])
                            }
                            _ => {
                                let op_str = match op {
                                    ComputeOp::Add => "+",
                                    ComputeOp::Subtract => "-",
                                    ComputeOp::Multiply => "*",
                                    ComputeOp::Divide => "/",
                                    ComputeOp::Power => "^",
                                    ComputeOp::Negate => "neg",
                                };
                                format!("[{}({})]", op_str, child_results.join(", "))
                            }
                        },
                        _ => unreachable!(),
                    };

                    results.push(result);
                }
            }
        }

        results.pop().unwrap_or_else(|| "<error>".to_string())
    }
}

impl DisplayNode<RuleId> for crate::Pool {
    fn display_with_children(&self, rule_id: RuleId) -> String {
        let rule = &self[rule_id];
        format!(
            "{}: {} => {}",
            self.display_name(rule.name),
            self.display_with_children(rule.pattern),
            self.display_with_children(rule.action)
        )
    }
}

impl crate::Pool {
    pub fn display_name(&self, name_id: NameId) -> String {
        if let Some(name) = self.get(name_id) {
            name.clone()
        } else {
            format!("name_{}", name_id.0)
        }
    }

    pub fn display_function(&self, fun_id: FunctionId) -> String {
        if let Some(function) = self.get(fun_id) {
            match function {
                Function::Add => "+".to_string(),
                Function::Subtract => "-".to_string(),
                Function::Multiply => "*".to_string(),
                Function::Divide => "/".to_string(),
                Function::Power => "^".to_string(),
                Function::Negate => "neg".to_string(),
                Function::Plus => "+".to_string(),
                Function::Custom(name_id) => self.display_name(*name_id),
            }
        } else {
            format!("func_{}", fun_id.0)
        }
    }
}
