use crate::{Action, ActionId, ComputeOp, Location, Pool};
use pest::Parser;
use pest::iterators::Pair;

use crate::parser::compute_parser::{ComputeParser, Rule};

pub fn parse_compute_expr(input: &str, pool: &mut Pool) -> Result<ActionId, String> {
    let pairs = ComputeParser::parse(Rule::compute_inner, input)
        .map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs.into_iter().next().unwrap();
    parse_compute_pair(pair, pool)
}

pub fn parse_compute_pair(pair: Pair<Rule>, pool: &mut Pool) -> Result<ActionId, String> {
    let span = pair.as_span();
    let location = Location::new(span.start(), span.end());

    match pair.as_rule() {
        Rule::compute_inner => parse_compute_pair(pair.into_inner().next().unwrap(), pool),
        Rule::compute_sum => {
            let mut inner = pair.into_inner();
            let mut action = parse_compute_pair(inner.next().unwrap(), pool)?;

            while let Some(op) = inner.next() {
                let right = parse_compute_pair(inner.next().unwrap(), pool)?;
                let compute_op = match op.as_str() {
                    "+" => ComputeOp::Add,
                    "-" => ComputeOp::Subtract,
                    _ => return Err(format!("Unknown compute operator: {}", op.as_str())),
                };

                let new_node_pos = pool.actions.len();
                let first_child_pos = if action.0 < right.0 {
                    action.0
                } else {
                    right.0
                };
                let last = new_node_pos - first_child_pos;

                action = pool.add_action_with_location(
                    Action::Compute {
                        op: compute_op,
                        last,
                        arity: 2,
                    },
                    location.clone(),
                );
            }
            Ok(action)
        }
        Rule::compute_product => {
            let mut inner = pair.into_inner();
            let mut action = parse_compute_pair(inner.next().unwrap(), pool)?;

            while let Some(op) = inner.next() {
                let right = parse_compute_pair(inner.next().unwrap(), pool)?;
                let compute_op = match op.as_str() {
                    "*" => ComputeOp::Multiply,
                    "/" => ComputeOp::Divide,
                    _ => return Err(format!("Unknown compute operator: {}", op.as_str())),
                };

                let new_node_pos = pool.actions.len();
                let first_child_pos = if action.0 < right.0 {
                    action.0
                } else {
                    right.0
                };
                let last = new_node_pos - first_child_pos;

                action = pool.add_action_with_location(
                    Action::Compute {
                        op: compute_op,
                        last,
                        arity: 2,
                    },
                    location.clone(),
                );
            }
            Ok(action)
        }
        Rule::compute_power => {
            let mut inner = pair.into_inner();
            let mut action = parse_compute_pair(inner.next().unwrap(), pool)?;

            for right_pair in inner {
                let right = parse_compute_pair(right_pair, pool)?;

                let new_node_pos = pool.actions.len();
                let first_child_pos = if action.0 < right.0 {
                    action.0
                } else {
                    right.0
                };
                let last = new_node_pos - first_child_pos;

                action = pool.add_action_with_location(
                    Action::Compute {
                        op: ComputeOp::Power,
                        last,
                        arity: 2,
                    },
                    location.clone(),
                );
            }
            Ok(action)
        }
        Rule::compute_value => {
            let inner = pair.into_inner();
            let mut ops = Vec::new();
            let mut atom_pair = None;

            for p in inner {
                match p.as_rule() {
                    Rule::unary_op => match p.as_str() {
                        "-" => ops.push(ComputeOp::Negate),
                        "+" => {}
                        _ => {}
                    },
                    _ => {
                        atom_pair = Some(p);
                        break;
                    }
                }
            }

            let mut action = parse_compute_pair(atom_pair.unwrap(), pool)?;

            for compute_op in ops.into_iter().rev() {
                action = pool.add_action_with_location(
                    Action::Compute {
                        op: compute_op,
                        last: pool.actions.len() - action.0,
                        arity: 1,
                    },
                    location.clone(),
                );
            }

            Ok(action)
        }
        Rule::number => {
            let num = pair
                .as_str()
                .parse::<i32>()
                .map_err(|_| format!("Invalid number: {}", pair.as_str()))?;
            Ok(pool.add_action_with_location(Action::Number(num), location))
        }
        Rule::variable => {
            let var_name = pair.as_str().to_string();
            let var_id = pool.intern_string(var_name);
            Ok(pool.add_action_with_location(Action::Variable(var_id), location))
        }
        _ => Err(format!("Unexpected compute rule: {:?}", pair.as_rule())),
    }
}
