use crate::display::DisplayNode;
use crate::idx::*;
use crate::pool::Pool;
use std::collections::HashMap;

impl Pool {
    pub fn add_transformation(&mut self, from: ExprId, to: ExprId, rule: RuleId) {
        println!(
            "ADD_TRANSFORMATION: {:?} -> {:?} via rule {:?}",
            from, to, rule
        );
        println!("  From expr: {}", self.display_with_children(from));
        println!("  To expr: {}", self.display_with_children(to));

        self.update_equivalence_groups(from);
        self.update_equivalence_groups(to);

        let from_group = self.get_equivalence_group(from).unwrap();
        let to_group = self.get_equivalence_group(to).unwrap();

        self.outgoing.entry(from).or_default().push((to, rule));

        self.incoming.entry(to).or_default().push((from, rule));

        self.by_rule.entry(rule).or_default().push((from, to));

        let equiv_outgoing = self.equivalence_outgoing.entry(from_group).or_default();
        if !equiv_outgoing.contains(&(to_group, rule)) {
            equiv_outgoing.push((to_group, rule));
        }

        let equiv_incoming = self.equivalence_incoming.entry(to_group).or_default();
        if !equiv_incoming.contains(&(from_group, rule)) {
            equiv_incoming.push((from_group, rule));
        }

        println!("  Total outgoing edges now: {}", self.outgoing.len());
        println!("  Outgoing from {}: {:?}", from, self.outgoing.get(&from));
    }

    pub fn get_outgoing_transformations(&self, expr: ExprId) -> Option<&Vec<(ExprId, RuleId)>> {
        self.outgoing.get(&expr)
    }

    pub fn get_incoming_transformations(&self, expr: ExprId) -> Option<&Vec<(ExprId, RuleId)>> {
        self.incoming.get(&expr)
    }

    pub fn get_rule_applications(&self, rule: RuleId) -> Option<&Vec<(ExprId, ExprId)>> {
        self.by_rule.get(&rule)
    }

    pub fn find_transformation_path(&self, from: ExprId, to: ExprId) -> Option<Vec<RuleId>> {
        use std::collections::{HashSet, VecDeque};

        if from == to {
            return Some(Vec::new());
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(from);
        visited.insert(from);

        while let Some(current) = queue.pop_front() {
            if let Some(outgoing) = self.get_outgoing_transformations(current) {
                for &(next_expr, rule) in outgoing {
                    if !visited.contains(&next_expr) {
                        visited.insert(next_expr);
                        parent.insert(next_expr, (current, rule));
                        queue.push_back(next_expr);

                        if next_expr == to {
                            let mut path = Vec::new();
                            let mut current_node = to;

                            while let Some((prev_node, rule)) = parent.get(&current_node) {
                                path.push(*rule);
                                current_node = *prev_node;
                            }

                            path.reverse();
                            return Some(path);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_derivation_history(&self, expr: ExprId) -> Vec<RuleId> {
        let mut history = Vec::new();
        let mut current = expr;

        while let Some(incoming) = self.get_incoming_transformations(current) {
            if let Some(&(prev_expr, rule)) = incoming.first() {
                history.push(rule);
                current = prev_expr;
            } else {
                break;
            }
        }

        history.reverse();
        history
    }

    pub fn bfs<T, F, G>(&self, start: T, mut get_neighbors: F, mut visit: G)
    where
        T: Copy + std::hash::Hash + Eq,
        F: FnMut(&Self, T) -> Vec<T>,
        G: FnMut(T),
    {
        use std::collections::{HashSet, VecDeque};

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(current) = queue.pop_front() {
            visit(current);

            for neighbor in get_neighbors(self, current) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
    }

    pub fn dfs<T, F, G>(&self, start: T, mut get_neighbors: F, mut visit: G)
    where
        T: Copy + std::hash::Hash + Eq,
        F: FnMut(&Self, T) -> Vec<T>,
        G: FnMut(T),
    {
        use std::collections::HashSet;

        let mut visited = HashSet::new();
        let mut stack = vec![start];

        while let Some(current) = stack.pop() {
            if !visited.contains(&current) {
                visited.insert(current);
                visit(current);

                for neighbor in get_neighbors(self, current) {
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }
        }
    }

    pub fn detect_cycles(&self) -> Vec<Vec<ExprId>> {
        use std::collections::HashSet;

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles = Vec::new();

        let mut all_nodes = HashSet::new();
        for &node in self.outgoing.keys() {
            all_nodes.insert(node);
        }
        for &node in self.incoming.keys() {
            all_nodes.insert(node);
        }

        for &node in &all_nodes {
            if !visited.contains(&node) {
                let mut path = Vec::new();
                self.dfs_cycle_detection(
                    node,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_cycle_detection(
        &self,
        node: ExprId,
        visited: &mut std::collections::HashSet<ExprId>,
        rec_stack: &mut std::collections::HashSet<ExprId>,
        path: &mut Vec<ExprId>,
        cycles: &mut Vec<Vec<ExprId>>,
    ) {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(outgoing) = self.get_outgoing_transformations(node) {
            for &(next_node, _rule) in outgoing {
                if !visited.contains(&next_node) {
                    self.dfs_cycle_detection(next_node, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(&next_node) {
                    if let Some(cycle_start) = path.iter().position(|&x| x == next_node) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(&node);
    }

    pub fn find_all_reachable(&self, start: ExprId) -> std::collections::HashSet<ExprId> {
        use std::collections::{HashSet, VecDeque};

        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        reachable.insert(start);

        while let Some(current) = queue.pop_front() {
            if let Some(outgoing) = self.get_outgoing_transformations(current) {
                for &(next_expr, _rule) in outgoing {
                    if !reachable.contains(&next_expr) {
                        reachable.insert(next_expr);
                        queue.push_back(next_expr);
                    }
                }
            }
        }

        reachable
    }

    pub fn find_strongly_connected_components(&self) -> Vec<Vec<ExprId>> {
        use std::collections::HashSet;

        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut all_nodes = HashSet::new();

        for &node in self.outgoing.keys() {
            all_nodes.insert(node);
        }
        for &node in self.incoming.keys() {
            all_nodes.insert(node);
        }

        for &node in &all_nodes {
            if !visited.contains(&node) {
                self.dfs_fill_order(node, &mut visited, &mut stack);
            }
        }

        visited.clear();
        let mut components = Vec::new();

        while let Some(node) = stack.pop() {
            if !visited.contains(&node) {
                let mut component = Vec::new();
                self.dfs_transpose(node, &mut visited, &mut component);
                if !component.is_empty() {
                    components.push(component);
                }
            }
        }

        components
    }

    fn dfs_fill_order(
        &self,
        node: ExprId,
        visited: &mut std::collections::HashSet<ExprId>,
        stack: &mut Vec<ExprId>,
    ) {
        visited.insert(node);

        if let Some(outgoing) = self.get_outgoing_transformations(node) {
            for &(next_node, _rule) in outgoing {
                if !visited.contains(&next_node) {
                    self.dfs_fill_order(next_node, visited, stack);
                }
            }
        }

        stack.push(node);
    }

    fn dfs_transpose(
        &self,
        node: ExprId,
        visited: &mut std::collections::HashSet<ExprId>,
        component: &mut Vec<ExprId>,
    ) {
        visited.insert(node);
        component.push(node);

        if let Some(incoming) = self.get_incoming_transformations(node) {
            for &(prev_node, _rule) in incoming {
                if !visited.contains(&prev_node) {
                    self.dfs_transpose(prev_node, visited, component);
                }
            }
        }
    }

    pub fn has_infinite_derivation_potential(&self, expr: ExprId) -> bool {
        let reachable = self.find_all_reachable(expr);
        let cycles = self.detect_cycles();

        for cycle in cycles {
            for &cycle_node in &cycle {
                if reachable.contains(&cycle_node) {
                    return true;
                }
            }
        }

        false
    }

    pub fn get_equivalence_group(&self, expr_id: ExprId) -> Option<EquivalenceGroupId> {
        self.expr_to_group.get(&expr_id).copied()
    }

    pub fn get_group_expressions(
        &self,
        group_id: EquivalenceGroupId,
    ) -> Option<&std::collections::HashSet<ExprId>> {
        self.equivalence_groups.get(group_id.0)
    }

    pub fn get_all_equivalence_groups(&self) -> &Vec<std::collections::HashSet<ExprId>> {
        &self.equivalence_groups
    }

    pub fn update_equivalence_groups(&mut self, new_expr: ExprId) {
        for (&existing_expr, &group_id) in &self.expr_to_group.clone() {
            if self.expr_eq(new_expr, existing_expr) {
                if let Some(group) = self.equivalence_groups.get_mut(group_id.0) {
                    group.insert(new_expr);
                    self.expr_to_group.insert(new_expr, group_id);
                    return;
                }
            }
        }

        let group_id = EquivalenceGroupId::new(self.equivalence_groups.len());
        let mut new_group = std::collections::HashSet::new();
        new_group.insert(new_expr);
        self.equivalence_groups.push(new_group);
        self.expr_to_group.insert(new_expr, group_id);
    }

    pub fn should_apply_rule(
        &self,
        from_group: EquivalenceGroupId,
        rule: RuleId,
        to_group: EquivalenceGroupId,
        current_chain: &[(EquivalenceGroupId, RuleId)],
    ) -> bool {
        if current_chain.len() >= self.max_chain_length {
            return false;
        }

        let proposed_step = (from_group, rule);

        for &(chain_group, _) in current_chain {
            if chain_group == to_group {
                return false;
            }
        }

        if current_chain.contains(&proposed_step) {
            return false;
        }

        let mut proposed_chain = current_chain.to_vec();
        proposed_chain.push(proposed_step);

        if self.blacklisted_chains.contains(&proposed_chain) {
            return false;
        }

        for i in 0..proposed_chain.len() {
            let suffix = &proposed_chain[i..];
            if self.blacklisted_chains.contains(suffix) {
                return false;
            }
        }

        true
    }

    pub fn start_rule_application_chain(&mut self, group_id: EquivalenceGroupId) {
        self.current_application_chains.insert(group_id, Vec::new());
    }

    pub fn extend_rule_application_chain(
        &mut self,
        group_id: EquivalenceGroupId,
        step: (EquivalenceGroupId, RuleId),
    ) {
        if let Some(chain) = self.current_application_chains.get_mut(&group_id) {
            chain.push(step);
        }
    }

    pub fn get_current_chain(
        &self,
        group_id: EquivalenceGroupId,
    ) -> Vec<(EquivalenceGroupId, RuleId)> {
        self.current_application_chains
            .get(&group_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn clear_application_chain(&mut self, group_id: EquivalenceGroupId) {
        self.current_application_chains.remove(&group_id);
    }

    pub fn detect_equivalence_cycles(&self) -> Vec<Vec<(EquivalenceGroupId, RuleId)>> {
        use std::collections::HashSet;

        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for &group_id in self.equivalence_outgoing.keys() {
            if !visited.contains(&group_id) {
                self.dfs_detect_equivalence_cycles(
                    group_id,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_detect_equivalence_cycles(
        &self,
        group_id: EquivalenceGroupId,
        visited: &mut std::collections::HashSet<EquivalenceGroupId>,
        rec_stack: &mut std::collections::HashSet<EquivalenceGroupId>,
        path: &mut Vec<(EquivalenceGroupId, RuleId)>,
        cycles: &mut Vec<Vec<(EquivalenceGroupId, RuleId)>>,
    ) {
        visited.insert(group_id);
        rec_stack.insert(group_id);

        if let Some(outgoing) = self.equivalence_outgoing.get(&group_id) {
            for &(next_group, rule) in outgoing {
                let edge = (group_id, rule);
                path.push(edge);

                if rec_stack.contains(&next_group) {
                    if let Some(cycle_start) = path.iter().position(|(g, _)| *g == next_group) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                } else if !visited.contains(&next_group) {
                    self.dfs_detect_equivalence_cycles(
                        next_group, visited, rec_stack, path, cycles,
                    );
                }

                path.pop();
            }
        }

        rec_stack.remove(&group_id);
    }

    pub fn detect_and_blacklist_cycles(&mut self) {
        let cycles = self.detect_equivalence_cycles();
        for cycle in cycles {
            if cycle.len() > 1 {
                println!("Blacklisting cycle pattern: {:?}", cycle);
                self.blacklisted_chains.insert(cycle);
            }
        }
    }
}
