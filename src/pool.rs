use crate::ast::*;
use crate::children::Children;
use crate::idx::*;
use std::collections::HashMap;
use std::ops::Index;

pub struct AncestorIterator<'a> {
    pool: &'a Pool,
    current: Option<ExprId>,
}

impl<'a> Iterator for AncestorIterator<'a> {
    type Item = ExprId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            self.current = self.pool.parent(current);
            Some(current)
        } else {
            None
        }
    }
}

pub struct SiblingIterator {
    children: std::vec::IntoIter<ExprId>,
    exclude: ExprId,
}

impl Iterator for SiblingIterator {
    type Item = ExprId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.children.next() {
                Some(child) if child != self.exclude => return Some(child),
                Some(_) => continue,
                None => return None,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub start: usize,
    pub end: usize,
}

impl Location {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn span(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug, Clone)]
pub enum Provenance {
    Parsed(Location),
    Rule {
        rule_id: RuleId,
        source_node: ExprId,
        source_location: Location,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    pub path: Vec<usize>,
}

impl Coordinate {
    pub fn root() -> Self {
        Self { path: vec![] }
    }

    pub fn child(&self, index: usize) -> Self {
        let mut path = self.path.clone();
        path.push(index);
        Self { path }
    }

    pub fn parent(&self) -> Option<Self> {
        if self.path.is_empty() {
            None
        } else {
            let mut path = self.path.clone();
            path.pop();
            Some(Self { path })
        }
    }
}

impl std::fmt::Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.path.is_empty() {
            write!(f, "[]")
        } else {
            write!(
                f,
                "[{}]",
                self.path
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pool {
    pub exprs: Vec<ExprNode>,
    pub expr_ends: std::collections::BTreeSet<usize>,
    pub names: Vec<String>,
    pub name_map: HashMap<String, NameId>,
    pub functions: Vec<Function>,
    pub function_map: HashMap<Function, FunctionId>,
    pub patterns: Vec<Pattern>,
    pub rules: Vec<Rule>,
    pub rulesets: Vec<Ruleset>,
    pub actions: Vec<Action>,
    pub locations: Vec<Provenance>,
    pub rule_locations: Vec<Location>,
    pub action_locations: Vec<Location>,

    pub outgoing: HashMap<ExprId, Vec<(ExprId, RuleId)>>,
    pub incoming: HashMap<ExprId, Vec<(ExprId, RuleId)>>,
    pub by_rule: HashMap<RuleId, Vec<(ExprId, ExprId)>>,

    pub equivalence_groups: Vec<std::collections::HashSet<ExprId>>,
    pub expr_to_group: HashMap<ExprId, EquivalenceGroupId>,

    pub equivalence_outgoing: HashMap<EquivalenceGroupId, Vec<(EquivalenceGroupId, RuleId)>>,
    pub equivalence_incoming: HashMap<EquivalenceGroupId, Vec<(EquivalenceGroupId, RuleId)>>,

    pub blacklisted_chains: std::collections::HashSet<Vec<(EquivalenceGroupId, RuleId)>>,

    pub max_chain_length: usize,
    pub current_application_chains: HashMap<EquivalenceGroupId, Vec<(EquivalenceGroupId, RuleId)>>,
}

impl Default for Pool {
    fn default() -> Self {
        Self::new()
    }
}

impl Pool {
    pub fn new() -> Self {
        let mut pool = Self {
            exprs: Vec::new(),
            expr_ends: std::collections::BTreeSet::new(),
            names: Vec::new(),
            name_map: HashMap::new(),
            functions: Vec::new(),
            function_map: HashMap::new(),
            patterns: Vec::new(),
            rules: Vec::new(),
            rulesets: Vec::new(),
            actions: Vec::new(),
            locations: Vec::new(),
            rule_locations: Vec::new(),
            action_locations: Vec::new(),

            outgoing: HashMap::new(),
            incoming: HashMap::new(),
            by_rule: HashMap::new(),

            equivalence_groups: Vec::new(),
            expr_to_group: HashMap::new(),

            equivalence_outgoing: HashMap::new(),
            equivalence_incoming: HashMap::new(),
            blacklisted_chains: std::collections::HashSet::new(),

            max_chain_length: 1024,
            current_application_chains: HashMap::new(),
        };

        pool.intern_function(Function::Add);
        pool.intern_function(Function::Subtract);
        pool.intern_function(Function::Multiply);
        pool.intern_function(Function::Divide);
        pool.intern_function(Function::Power);
        pool.intern_function(Function::Negate);
        pool.intern_function(Function::Plus);

        pool
    }

    pub fn add_expr(&mut self, node: ExprNode) -> ExprId {
        let id = self.exprs.len();
        self.exprs.push(node);
        ExprId::new(id)
    }

    pub fn add_expr_with_provenance(&mut self, node: ExprNode, provenance: Provenance) -> ExprId {
        let id = self.exprs.len();
        self.exprs.push(node);
        self.locations.push(provenance);
        ExprId::new(id)
    }

    pub fn mark_expr_end(&mut self, expr_id: ExprId) {
        self.expr_ends.insert(expr_id.0);
    }

    pub fn find_root(&self, expr_id: ExprId) -> Option<ExprId> {
        self.expr_ends
            .range(expr_id.0..)
            .next()
            .map(|&end| ExprId(end))
    }

    pub fn is_root(&self, expr_id: ExprId) -> bool {
        self.expr_ends.contains(&expr_id.0)
    }

    pub fn get_all_roots(&self) -> impl Iterator<Item = ExprId> + '_ {
        self.expr_ends.iter().map(|&end| ExprId(end))
    }

    pub fn parent(&self, expr_id: ExprId) -> Option<ExprId> {
        let node_idx = expr_id.0;

        for candidate_idx in (node_idx + 1)..self.exprs.len() {
            let candidate_id = ExprId(candidate_idx);
            let candidate_node = &self.exprs[candidate_idx];

            match candidate_node {
                ExprNode::Call { last, arity, .. } | ExprNode::Struct { last, arity, .. } => {
                    if *arity > 0 {
                        let first_child_pos = candidate_idx - last;
                        if first_child_pos <= node_idx && node_idx <= candidate_idx {
                            return Some(candidate_id);
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn ancestors(&self, expr_id: ExprId) -> AncestorIterator {
        AncestorIterator {
            pool: self,
            current: Some(expr_id),
        }
    }

    pub fn siblings(&self, expr_id: ExprId) -> SiblingIterator {
        if let Some(parent_id) = self.parent(expr_id) {
            let children: Vec<_> = self.children(parent_id).collect();
            SiblingIterator {
                children: children.into_iter(),
                exclude: expr_id,
            }
        } else {
            SiblingIterator {
                children: vec![].into_iter(),
                exclude: expr_id,
            }
        }
    }

    pub fn intern_string(&mut self, name: String) -> NameId {
        if let Some(&id) = self.name_map.get(&name) {
            id
        } else {
            let id = NameId::new(self.names.len());
            self.names.push(name.clone());
            self.name_map.insert(name, id);
            id
        }
    }

    pub fn intern_function(&mut self, func: Function) -> FunctionId {
        if let Some(&id) = self.function_map.get(&func) {
            id
        } else {
            let id = FunctionId::new(self.functions.len());
            self.functions.push(func);
            self.function_map.insert(func, id);
            id
        }
    }

    pub fn get_provenance(&self, id: ExprId) -> Option<&Provenance> {
        self.locations.get(id.0)
    }

    pub fn add_pattern(&mut self, pattern: Pattern) -> PatternId {
        let id = self.patterns.len();
        self.patterns.push(pattern);
        PatternId::new(id)
    }

    pub fn add_rule(&mut self, rule: Rule) -> RuleId {
        let id = self.rules.len();
        self.rules.push(rule);
        RuleId::new(id)
    }

    pub fn add_rule_with_location(&mut self, rule: crate::ast::Rule, location: Location) -> RuleId {
        let rule_id = self.rules.len();
        self.rules.push(rule);
        self.rule_locations.push(location);
        RuleId::new(rule_id)
    }

    pub fn get_rule_location(&self, rule_id: RuleId) -> Option<&Location> {
        self.rule_locations.get(rule_id.0)
    }

    pub fn add_ruleset(&mut self, ruleset: crate::ast::Ruleset) -> RulesetId {
        let id = self.rulesets.len();
        self.rulesets.push(ruleset);
        RulesetId::new(id)
    }

    pub fn get_ruleset_rules(
        &self,
        ruleset_id: RulesetId,
    ) -> impl Iterator<Item = &crate::ast::Rule> {
        let start = self
            .rulesets
            .get(ruleset_id.0)
            .map(|r| r.rules_start)
            .unwrap_or(0);
        let end = self
            .rulesets
            .get(ruleset_id.0)
            .map(|r| r.rules_end)
            .unwrap_or(0);
        (start..end).filter_map(move |i| self.rules.get(i))
    }

    pub fn get_rules_len(&self) -> usize {
        self.rules.len()
    }

    pub fn get_ruleset_rule_count(&self, ruleset_id: RulesetId) -> usize {
        if let Some(ruleset) = self.rulesets.get(ruleset_id.0) {
            ruleset.rules_end - ruleset.rules_start
        } else {
            0
        }
    }

    pub fn add_action(&mut self, action: crate::ast::Action) -> ActionId {
        let id = self.actions.len();
        self.actions.push(action);
        ActionId::new(id)
    }

    pub fn add_action_with_location(
        &mut self,
        action: crate::ast::Action,
        location: Location,
    ) -> ActionId {
        let action_id = self.actions.len();
        self.actions.push(action);
        self.action_locations.push(location);
        ActionId::new(action_id)
    }

    pub fn get_action_location(&self, action_id: ActionId) -> Option<&Location> {
        self.action_locations.get(action_id.0)
    }

    pub fn reset(&mut self) {
        self.exprs.clear();
        self.expr_ends.clear();
        self.names.clear();
        self.name_map.clear();
        self.functions.clear();
        self.function_map.clear();
        self.patterns.clear();
        self.rules.clear();
        self.rulesets.clear();
        self.actions.clear();
        self.locations.clear();
        self.rule_locations.clear();
        self.action_locations.clear();

        self.outgoing.clear();
        self.incoming.clear();
        self.by_rule.clear();

        self.equivalence_groups.clear();
        self.expr_to_group.clear();

        self.equivalence_outgoing.clear();
        self.equivalence_incoming.clear();
        self.blacklisted_chains.clear();

        self.current_application_chains.clear();

        self.intern_function(Function::Add);
        self.intern_function(Function::Subtract);
        self.intern_function(Function::Multiply);
        self.intern_function(Function::Divide);
        self.intern_function(Function::Power);
        self.intern_function(Function::Negate);
        self.intern_function(Function::Plus);
    }

    pub fn get_children_vec(pool: &Pool, expr: ExprId) -> Vec<ExprId> {
        pool.children(expr).collect()
    }
}

impl Index<ExprId> for Pool {
    type Output = ExprNode;
    fn index(&self, id: ExprId) -> &Self::Output {
        &self.exprs[id.0]
    }
}

impl Index<NameId> for Pool {
    type Output = String;
    fn index(&self, id: NameId) -> &Self::Output {
        &self.names[id.0]
    }
}

impl Index<FunctionId> for Pool {
    type Output = Function;
    fn index(&self, id: FunctionId) -> &Self::Output {
        &self.functions[id.0]
    }
}

impl Index<RulesetId> for Pool {
    type Output = Ruleset;
    fn index(&self, id: RulesetId) -> &Self::Output {
        &self.rulesets[id.0]
    }
}

impl Index<PatternId> for Pool {
    type Output = Pattern;
    fn index(&self, id: PatternId) -> &Self::Output {
        &self.patterns[id.0]
    }
}

impl Index<RuleId> for Pool {
    type Output = Rule;
    fn index(&self, id: RuleId) -> &Self::Output {
        &self.rules[id.0]
    }
}

impl Index<ActionId> for Pool {
    type Output = Action;
    fn index(&self, id: ActionId) -> &Self::Output {
        &self.actions[id.0]
    }
}

use std::ops::Range;

impl Index<Range<ExprId>> for Pool {
    type Output = [ExprNode];
    fn index(&self, range: Range<ExprId>) -> &Self::Output {
        &self.exprs[range.start.0..range.end.0]
    }
}

impl Index<Range<ActionId>> for Pool {
    type Output = [Action];
    fn index(&self, range: Range<ActionId>) -> &Self::Output {
        &self.actions[range.start.0..range.end.0]
    }
}

impl Index<Range<PatternId>> for Pool {
    type Output = [Pattern];
    fn index(&self, range: Range<PatternId>) -> &Self::Output {
        &self.patterns[range.start.0..range.end.0]
    }
}

impl Index<Range<FunctionId>> for Pool {
    type Output = [Function];
    fn index(&self, range: Range<FunctionId>) -> &Self::Output {
        &self.functions[range.start.0..range.end.0]
    }
}

impl Index<Range<NameId>> for Pool {
    type Output = [String];
    fn index(&self, range: Range<NameId>) -> &Self::Output {
        &self.names[range.start.0..range.end.0]
    }
}

impl Index<Range<RuleId>> for Pool {
    type Output = [Rule];
    fn index(&self, range: Range<RuleId>) -> &Self::Output {
        &self.rules[range.start.0..range.end.0]
    }
}
