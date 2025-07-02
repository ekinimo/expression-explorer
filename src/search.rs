use crate::{DisplayNode, EquivalenceGroupId, ExprId, Pool, RuleId};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub struct SearchPath {
    pub steps: Vec<(ExprId, RuleId, ExprId)>,
    pub cost: f64,
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct SearchNode {
    pub expr_id: ExprId,
    pub group_id: EquivalenceGroupId,
    pub path: Vec<(ExprId, RuleId, ExprId)>,
    pub cost: f64,
    pub depth: usize,
    pub heuristic_score: f64,
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.cost.partial_cmp(&other.cost) == Some(Ordering::Equal)
    }
}

impl Eq for SearchNode {}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.cost.partial_cmp(&self.cost)
    }
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub max_depth: usize,
    pub max_nodes_explored: usize,
    pub beam_width: usize,
    pub random_walk_probability: f64,
    pub diversification_factor: f64,
    pub target_diversity: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            max_depth: 50,
            max_nodes_explored: 10000,
            beam_width: 10,
            random_walk_probability: 0.1,
            diversification_factor: 0.3,
            target_diversity: 100,
        }
    }
}

pub struct SearchEngine {
    config: SearchConfig,
    rng: fastrand::Rng,
}

impl SearchEngine {
    pub fn new(config: SearchConfig) -> Self {
        SearchEngine {
            config,
            rng: fastrand::Rng::new(),
        }
    }

    pub fn bounded_bfs(
        &mut self,
        pool: &mut Pool,
        start_expr: ExprId,
        target_expr: Option<ExprId>,
    ) -> Vec<SearchPath> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut paths = Vec::new();
        let mut nodes_explored = 0;

        let start_group = match pool.get_equivalence_group(start_expr) {
            Some(group) => group,
            None => return vec![], // No equivalence group found
        };
        queue.push_back(SearchNode {
            expr_id: start_expr,
            group_id: start_group,
            path: Vec::new(),
            cost: 0.0,
            depth: 0,
            heuristic_score: 0.0,
        });
        visited.insert(start_group);

        while let Some(current) = queue.pop_front() {
            nodes_explored += 1;

            if nodes_explored >= self.config.max_nodes_explored {
                break;
            }

            if current.depth >= self.config.max_depth {
                continue;
            }

            if let Some(target) = target_expr {
                if pool.expr_eq(current.expr_id, target) {
                    paths.push(SearchPath {
                        steps: current.path.clone(),
                        cost: current.cost,
                        length: current.depth,
                    });
                    continue;
                }
            }

            // First try existing transformations
            if let Some(outgoing) = pool.get_outgoing_transformations(current.expr_id) {
                for &(next_expr, rule) in outgoing {
                    let next_group = pool.get_equivalence_group(next_expr).unwrap();

                    if visited.contains(&next_group) {
                        continue;
                    }

                    let current_chain: Vec<_> = current
                        .path
                        .iter()
                        .map(|(from, rule, _)| (pool.get_equivalence_group(*from).unwrap(), *rule))
                        .collect();

                    if !pool.should_apply_rule(current.group_id, rule, next_group, &current_chain) {
                        continue;
                    }

                    let mut new_path = current.path.clone();
                    new_path.push((current.expr_id, rule, next_expr));

                    queue.push_back(SearchNode {
                        expr_id: next_expr,
                        group_id: next_group,
                        path: new_path,
                        cost: current.cost + 1.0,
                        depth: current.depth + 1,
                        heuristic_score: 0.0,
                    });

                    visited.insert(next_group);
                }
            }
            
            // Then apply rules to generate new expressions
            let matches = pool.find_matches(current.expr_id);
            for match_ in matches {
                if let Some(new_expr) = pool.apply_rule(&match_) {
                    pool.update_equivalence_groups(new_expr);
                    let new_group = match pool.get_equivalence_group(new_expr) {
                        Some(group) => group,
                        None => continue,
                    };
                    
                    if visited.contains(&new_group) {
                        continue;
                    }
                    
                    let current_chain: Vec<_> = current
                        .path
                        .iter()
                        .map(|(from, rule, _)| (pool.get_equivalence_group(*from).unwrap(), *rule))
                        .collect();

                    if !pool.should_apply_rule(current.group_id, match_.rule_id, new_group, &current_chain) {
                        continue;
                    }
                    
                    let mut new_path = current.path.clone();
                    new_path.push((current.expr_id, match_.rule_id, new_expr));
                    
                    queue.push_back(SearchNode {
                        expr_id: new_expr,
                        group_id: new_group,
                        path: new_path,
                        cost: current.cost + 1.0,
                        depth: current.depth + 1,
                        heuristic_score: 0.0,
                    });
                    
                    visited.insert(new_group);
                }
            }

            if target_expr.is_none() && current.depth > 0 {
                paths.push(SearchPath {
                    steps: current.path.clone(),
                    cost: current.cost,
                    length: current.depth,
                });
            }
        }

        paths
    }

    pub fn bounded_dijkstra(
        &mut self,
        pool: &mut Pool,
        start_expr: ExprId,
        target_expr: Option<ExprId>,
        cost_fn: impl Fn(&Pool, RuleId, ExprId, ExprId) -> f64,
    ) -> Vec<SearchPath> {
        let mut heap = BinaryHeap::new();
        let mut distances: HashMap<EquivalenceGroupId, f64> = HashMap::new();
        let mut paths: HashMap<EquivalenceGroupId, Vec<(ExprId, RuleId, ExprId)>> = HashMap::new();
        let mut result_paths = Vec::new();
        let mut nodes_explored = 0;

        let start_group = match pool.get_equivalence_group(start_expr) {
            Some(group) => group,
            None => return vec![], // No equivalence group found
        };
        heap.push(SearchNode {
            expr_id: start_expr,
            group_id: start_group,
            path: Vec::new(),
            cost: 0.0,
            depth: 0,
            heuristic_score: 0.0,
        });
        distances.insert(start_group, 0.0);
        paths.insert(start_group, Vec::new());

        while let Some(current) = heap.pop() {
            nodes_explored += 1;

            if nodes_explored >= self.config.max_nodes_explored {
                break;
            }

            if current.depth >= self.config.max_depth {
                continue;
            }

            if let Some(&best_dist) = distances.get(&current.group_id) {
                if current.cost > best_dist {
                    continue;
                }
            }

            if let Some(target) = target_expr {
                if pool.expr_eq(current.expr_id, target) {
                    result_paths.push(SearchPath {
                        steps: current.path.clone(),
                        cost: current.cost,
                        length: current.depth,
                    });
                    continue;
                }
            }

            // First try existing transformations
            if let Some(outgoing) = pool.get_outgoing_transformations(current.expr_id) {
                for &(next_expr, rule) in outgoing {
                    let next_group = pool.get_equivalence_group(next_expr).unwrap();

                    let current_chain: Vec<_> = current
                        .path
                        .iter()
                        .map(|(from, rule, _)| (pool.get_equivalence_group(*from).unwrap(), *rule))
                        .collect();

                    if !pool.should_apply_rule(current.group_id, rule, next_group, &current_chain) {
                        continue;
                    }

                    let edge_cost = cost_fn(pool, rule, current.expr_id, next_expr);
                    let new_cost = current.cost + edge_cost;

                    if let Some(&best_dist) = distances.get(&next_group) {
                        if new_cost >= best_dist {
                            continue;
                        }
                    }

                    let mut new_path = current.path.clone();
                    new_path.push((current.expr_id, rule, next_expr));

                    distances.insert(next_group, new_cost);
                    paths.insert(next_group, new_path.clone());

                    heap.push(SearchNode {
                        expr_id: next_expr,
                        group_id: next_group,
                        path: new_path,
                        cost: new_cost,
                        depth: current.depth + 1,
                        heuristic_score: 0.0,
                    });
                }
            }
            
            // Then apply rules to generate new expressions
            let matches = pool.find_matches(current.expr_id);
            for match_ in matches {
                if let Some(new_expr) = pool.apply_rule(&match_) {
                    pool.update_equivalence_groups(new_expr);
                    let new_group = match pool.get_equivalence_group(new_expr) {
                        Some(group) => group,
                        None => continue,
                    };
                    
                    let current_chain: Vec<_> = current
                        .path
                        .iter()
                        .map(|(from, rule, _)| (pool.get_equivalence_group(*from).unwrap(), *rule))
                        .collect();

                    if !pool.should_apply_rule(current.group_id, match_.rule_id, new_group, &current_chain) {
                        continue;
                    }

                    let edge_cost = cost_fn(pool, match_.rule_id, current.expr_id, new_expr);
                    let new_cost = current.cost + edge_cost;

                    if let Some(&best_dist) = distances.get(&new_group) {
                        if new_cost >= best_dist {
                            continue;
                        }
                    }

                    let mut new_path = current.path.clone();
                    new_path.push((current.expr_id, match_.rule_id, new_expr));

                    distances.insert(new_group, new_cost);
                    paths.insert(new_group, new_path.clone());

                    heap.push(SearchNode {
                        expr_id: new_expr,
                        group_id: new_group,
                        path: new_path,
                        cost: new_cost,
                        depth: current.depth + 1,
                        heuristic_score: 0.0,
                    });
                }
            }
        }

        if target_expr.is_none() {
            for (group_id, path) in paths {
                if !path.is_empty() {
                    let cost = distances.get(&group_id).copied().unwrap_or(0.0);
                    result_paths.push(SearchPath {
                        steps: path.clone(),
                        cost,
                        length: path.len(),
                    });
                }
            }
        }

        result_paths
    }

    pub fn heuristic_search(
        &mut self,
        pool: &mut Pool,
        start_expr: ExprId,
        target_expr: ExprId,
        heuristic_fn: impl Fn(&Pool, ExprId, ExprId) -> f64,
        cost_fn: impl Fn(&Pool, RuleId, ExprId, ExprId) -> f64,
    ) -> Option<SearchPath> {
        let mut heap = BinaryHeap::new();
        let mut g_score: HashMap<EquivalenceGroupId, f64> = HashMap::new();
        let mut f_score: HashMap<EquivalenceGroupId, f64> = HashMap::new();
        let mut came_from: HashMap<EquivalenceGroupId, (ExprId, RuleId, ExprId)> = HashMap::new();
        let mut nodes_explored = 0;

        let start_group = match pool.get_equivalence_group(start_expr) {
            Some(group) => group,
            None => return None, // No equivalence group found
        };
        let target_group = match pool.get_equivalence_group(target_expr) {
            Some(group) => group,
            None => return None, // No equivalence group found
        };

        let h_start = heuristic_fn(pool, start_expr, target_expr);
        g_score.insert(start_group, 0.0);
        f_score.insert(start_group, h_start);

        heap.push(SearchNode {
            expr_id: start_expr,
            group_id: start_group,
            path: Vec::new(),
            cost: h_start, 
            depth: 0,
            heuristic_score: h_start,
        });

        while let Some(current) = heap.pop() {
            nodes_explored += 1;

            if nodes_explored >= self.config.max_nodes_explored {
                break;
            }

            if current.depth >= self.config.max_depth {
                continue;
            }

            if current.group_id == target_group {
                let mut path = Vec::new();
                let mut current_group = target_group;

                while let Some(&(from, rule, to)) = came_from.get(&current_group) {
                    path.push((from, rule, to));
                    current_group = pool.get_equivalence_group(from).unwrap();
                }

                path.reverse();
                return Some(SearchPath {
                    steps: path.clone(),
                    cost: g_score.get(&target_group).copied().unwrap_or(0.0),
                    length: path.len(),
                });
            }

            if let Some(outgoing) = pool.get_outgoing_transformations(current.expr_id) {
                for &(next_expr, rule) in outgoing {
                    let next_group = pool.get_equivalence_group(next_expr).unwrap();

                    let current_chain: Vec<_> = current
                        .path
                        .iter()
                        .map(|(from, rule, _)| (pool.get_equivalence_group(*from).unwrap(), *rule))
                        .collect();

                    if !pool.should_apply_rule(current.group_id, rule, next_group, &current_chain) {
                        continue;
                    }

                    let tentative_g = g_score
                        .get(&current.group_id)
                        .copied()
                        .unwrap_or(f64::INFINITY)
                        + cost_fn(pool, rule, current.expr_id, next_expr);

                    if tentative_g < g_score.get(&next_group).copied().unwrap_or(f64::INFINITY) {
                        came_from.insert(next_group, (current.expr_id, rule, next_expr));
                        g_score.insert(next_group, tentative_g);

                        let h = heuristic_fn(pool, next_expr, target_expr);
                        let f = tentative_g + h;
                        f_score.insert(next_group, f);

                        let mut new_path = current.path.clone();
                        new_path.push((current.expr_id, rule, next_expr));

                        heap.push(SearchNode {
                            expr_id: next_expr,
                            group_id: next_group,
                            path: new_path,
                            cost: f,
                            depth: current.depth + 1,
                            heuristic_score: h,
                        });
                    }
                }
            }
        }

        None
    }

    pub fn random_search(
        &mut self,
        pool: &mut Pool,
        start_expr: ExprId,
        num_walks: usize,
    ) -> Vec<SearchPath> {
        let mut paths = Vec::new();

        for _ in 0..num_walks {
            let mut current_expr = start_expr;
            let mut path = Vec::new();
            let mut visited_groups = HashSet::new();

            let start_group = match pool.get_equivalence_group(start_expr) {
                Some(group) => group,
                None => continue, // Skip this walk if no equivalence group
            };
            visited_groups.insert(start_group);

            for _depth in 0..self.config.max_depth {
                let mut all_moves = Vec::new();
                
                // First collect existing transformations
                if let Some(outgoing) = pool.get_outgoing_transformations(current_expr) {
                    for &(next_expr, rule) in outgoing {
                        let current_group = pool.get_equivalence_group(current_expr).unwrap();
                        let next_group = pool.get_equivalence_group(next_expr).unwrap();

                        if visited_groups.contains(&next_group) {
                            continue;
                        }

                        let current_chain: Vec<_> = path
                            .iter()
                            .map(|(from, rule, _)| {
                                (pool.get_equivalence_group(*from).unwrap(), *rule)
                            })
                            .collect();

                        if pool.should_apply_rule(current_group, rule, next_group, &current_chain) {
                            all_moves.push((next_expr, rule));
                        }
                    }
                }
                
                // Then apply rules to generate new moves
                let matches = pool.find_matches(current_expr);
                for match_ in matches {
                    if let Some(new_expr) = pool.apply_rule(&match_) {
                        pool.update_equivalence_groups(new_expr);
                        let current_group = pool.get_equivalence_group(current_expr).unwrap();
                        let new_group = match pool.get_equivalence_group(new_expr) {
                            Some(group) => group,
                            None => continue,
                        };

                        if visited_groups.contains(&new_group) {
                            continue;
                        }

                        let current_chain: Vec<_> = path
                            .iter()
                            .map(|(from, rule, _)| {
                                (pool.get_equivalence_group(*from).unwrap(), *rule)
                            })
                            .collect();

                        if pool.should_apply_rule(current_group, match_.rule_id, new_group, &current_chain) {
                            all_moves.push((new_expr, match_.rule_id));
                        }
                    }
                }

                if all_moves.is_empty() {
                    break;
                }

                let &(next_expr, rule) = &all_moves[self.rng.usize(0..all_moves.len())];
                let next_group = pool.get_equivalence_group(next_expr).unwrap();

                path.push((current_expr, rule, next_expr));
                visited_groups.insert(next_group);
                current_expr = next_expr;

                if self.rng.f64() < self.config.random_walk_probability {
                    break;
                }
            }

            if !path.is_empty() {
                paths.push(SearchPath {
                    steps: path.clone(),
                    cost: path.len() as f64,
                    length: path.len(),
                });
            }
        }

        paths
    }

    pub fn beam_search(
        &mut self,
        pool: &mut Pool,
        start_expr: ExprId,
        evaluation_fn: impl Fn(&Pool, ExprId, &[ExprId]) -> f64,
    ) -> Vec<SearchPath> {
        let mut current_beam = Vec::new();
        let mut all_paths = Vec::new();

        let start_group = match pool.get_equivalence_group(start_expr) {
            Some(group) => group,
            None => return vec![], // No equivalence group found
        };
        current_beam.push(SearchNode {
            expr_id: start_expr,
            group_id: start_group,
            path: Vec::new(),
            cost: 0.0,
            depth: 0,
            heuristic_score: evaluation_fn(pool, start_expr, &[start_expr]),
        });

        for depth in 0..self.config.max_depth {
            let mut next_beam = Vec::new();
            let mut explored_groups = HashSet::new();

            for current in &current_beam {
                if let Some(outgoing) = pool.get_outgoing_transformations(current.expr_id) {
                    for &(next_expr, rule) in outgoing {
                        let next_group = pool.get_equivalence_group(next_expr).unwrap();

                        if explored_groups.contains(&next_group) {
                            continue;
                        }

                        let current_chain: Vec<_> = current
                            .path
                            .iter()
                            .map(|(from, rule, _)| {
                                (pool.get_equivalence_group(*from).unwrap(), *rule)
                            })
                            .collect();

                        if !pool.should_apply_rule(
                            current.group_id,
                            rule,
                            next_group,
                            &current_chain,
                        ) {
                            continue;
                        }

                        let mut new_path = current.path.clone();
                        new_path.push((current.expr_id, rule, next_expr));

                        let beam_exprs: Vec<_> = current_beam.iter().map(|n| n.expr_id).collect();
                        let score = evaluation_fn(pool, next_expr, &beam_exprs);

                        next_beam.push(SearchNode {
                            expr_id: next_expr,
                            group_id: next_group,
                            path: new_path.clone(),
                            cost: current.cost + 1.0,
                            depth: depth + 1,
                            heuristic_score: score,
                        });

                        explored_groups.insert(next_group);

                        all_paths.push(SearchPath {
                            steps: new_path,
                            cost: current.cost + 1.0,
                            length: depth + 1,
                        });
                    }
                }
            }

            if next_beam.is_empty() {
                break;
            }

            next_beam.sort_by(|a, b| {
                b.heuristic_score
                    .partial_cmp(&a.heuristic_score)
                    .unwrap_or(Ordering::Equal)
            });
            next_beam.truncate(self.config.beam_width);

            current_beam = next_beam;
        }

        all_paths
    }

    pub fn combined_search(
        &mut self,
        pool: &mut Pool,
        start_expr: ExprId,
        target_expr: Option<ExprId>,
    ) -> Vec<SearchPath> {
        let mut all_paths = Vec::new();

        let mut bfs_paths = self.bounded_bfs(pool, start_expr, target_expr);
        all_paths.append(&mut bfs_paths);

        let unit_cost = |_pool: &Pool, _rule: RuleId, _from: ExprId, _to: ExprId| 1.0;
        let mut dijkstra_paths = self.bounded_dijkstra(pool, start_expr, target_expr, unit_cost);
        all_paths.append(&mut dijkstra_paths);

        let mut random_paths = self.random_search(pool, start_expr, 20);
        all_paths.append(&mut random_paths);

        let complexity_eval = |pool: &Pool, expr: ExprId, _beam: &[ExprId]| {
            pool.display_with_children(expr).len() as f64
        };
        let mut beam_paths = self.beam_search(pool, start_expr, complexity_eval);
        all_paths.append(&mut beam_paths);

        self.deduplicate_and_rank_paths(pool, all_paths)
    }

    fn deduplicate_and_rank_paths(
        &self,
        pool: &Pool,
        mut paths: Vec<SearchPath>,
    ) -> Vec<SearchPath> {
        let mut seen_end_states = HashSet::new();
        paths.retain(|path| {
            if let Some((_, _, end_expr)) = path.steps.last() {
                let end_group = pool.get_equivalence_group(*end_expr).unwrap();
                seen_end_states.insert(end_group)
            } else {
                false
            }
        });

        paths.sort_by(|a, b| {
            let len_cmp = a.length.cmp(&b.length);
            if len_cmp != Ordering::Equal {
                return len_cmp;
            }

            a.cost.partial_cmp(&b.cost).unwrap_or(Ordering::Equal)
        });

        paths.truncate(self.config.target_diversity);
        paths
    }
}

pub mod heuristics {
    use super::*;

    pub fn complexity_distance(pool: &Pool, from: ExprId, to: ExprId) -> f64 {
        let from_complexity = pool.display_with_children(from).len() as f64;
        let to_complexity = pool.display_with_children(to).len() as f64;
        (from_complexity - to_complexity).abs()
    }

    pub fn depth_distance(pool: &Pool, from: ExprId, to: ExprId) -> f64 {
        let from_depth = pool.display_with_children(from).matches('(').count() as f64;
        let to_depth = pool.display_with_children(to).matches('(').count() as f64;
        (from_depth - to_depth).abs()
    }

    pub fn edit_distance(pool: &Pool, from: ExprId, to: ExprId) -> f64 {
        let from_str = pool.display_with_children(from);
        let to_str = pool.display_with_children(to);
        levenshtein_distance(&from_str, &to_str) as f64
    }
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let mut prev_row: Vec<usize> = (0..=v2.len()).collect();

    for (i, c1) in v1.iter().enumerate() {
        let mut curr_row = vec![i + 1];
        for (j, c2) in v2.iter().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            curr_row.push(std::cmp::min(
                std::cmp::min(curr_row[j] + 1, prev_row[j + 1] + 1),
                prev_row[j] + cost,
            ));
        }
        prev_row = curr_row;
    }

    prev_row[v2.len()]
}
