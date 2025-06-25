use crate::{Action, ActionId, Function, Location, Pool};
use pest::Parser;
use pest::iterators::Pair;

use crate::parser::action_parser::{ActionParser, Rule};
use crate::parser::compute::parse_compute_expr;

pub fn parse_action(input: &str, pool: &mut Pool) -> Result<ActionId, String> {
    let pairs =
        ActionParser::parse(Rule::action, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs.into_iter().next().unwrap();
    parse_action_pair(pair, pool)
}

pub fn parse_action_pair(pair: Pair<Rule>, pool: &mut Pool) -> Result<ActionId, String> {
    let span = pair.as_span();
    let location = Location::new(span.start(), span.end());

    match pair.as_rule() {
        Rule::action => parse_action_pair(pair.into_inner().next().unwrap(), pool),
        Rule::action_sum => {
            let mut inner = pair.into_inner();
            let mut action = parse_action_pair(inner.next().unwrap(), pool)?;

            while let Some(op) = inner.next() {
                let right = parse_action_pair(inner.next().unwrap(), pool)?;
                let func = match op.as_str() {
                    "+" => Function::Add,
                    "-" => Function::Subtract,
                    _ => return Err(format!("Unknown operator: {}", op.as_str())),
                };
                let fun_id = pool.intern_function(func);

                let new_node_pos = pool.actions.len();
                let first_child_pos = if action.0 < right.0 {
                    action.0
                } else {
                    right.0
                };
                let last = new_node_pos - first_child_pos;

                action = pool.add_action_with_location(
                    Action::Call {
                        fun: fun_id,
                        last,
                        arity: 2,
                    },
                    location.clone(),
                );
            }
            Ok(action)
        }
        Rule::action_product => {
            let mut inner = pair.into_inner();
            let mut action = parse_action_pair(inner.next().unwrap(), pool)?;

            while let Some(op) = inner.next() {
                let right = parse_action_pair(inner.next().unwrap(), pool)?;
                let func = match op.as_str() {
                    "*" => Function::Multiply,
                    "/" => Function::Divide,
                    _ => return Err(format!("Unknown operator: {}", op.as_str())),
                };
                let fun_id = pool.intern_function(func);

                let new_node_pos = pool.actions.len();
                let first_child_pos = if action.0 < right.0 {
                    action.0
                } else {
                    right.0
                };
                let last = new_node_pos - first_child_pos;

                action = pool.add_action_with_location(
                    Action::Call {
                        fun: fun_id,
                        last,
                        arity: 2,
                    },
                    location.clone(),
                );
            }
            Ok(action)
        }
        Rule::action_power => {
            let mut inner = pair.into_inner();
            let mut action = parse_action_pair(inner.next().unwrap(), pool)?;

            for right_pair in inner {
                let right = parse_action_pair(right_pair, pool)?;
                let fun_id = pool.intern_function(Function::Power);

                let new_node_pos = pool.actions.len();
                let first_child_pos = if action.0 < right.0 {
                    action.0
                } else {
                    right.0
                };
                let last = new_node_pos - first_child_pos;

                action = pool.add_action_with_location(
                    Action::Call {
                        fun: fun_id,
                        last,
                        arity: 2,
                    },
                    location.clone(),
                );
            }
            Ok(action)
        }
        Rule::action_value => {
            let inner = pair.into_inner();
            let mut ops = Vec::new();
            let mut atom_pair = None;

            for p in inner {
                match p.as_rule() {
                    Rule::unary_op => match p.as_str() {
                        "-" => ops.push(Function::Negate),
                        "+" => ops.push(Function::Plus),
                        _ => {}
                    },
                    _ => {
                        atom_pair = Some(p);
                        break;
                    }
                }
            }

            let mut action = parse_action_pair(atom_pair.unwrap(), pool)?;

            for func in ops.into_iter().rev() {
                let fun_id = pool.intern_function(func);

                action = pool.add_action_with_location(
                    Action::Call {
                        fun: fun_id,
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
        Rule::action_function_call => {
            let mut inner = pair.into_inner();
            let func_name = inner.next().unwrap().as_str().to_string();
            let name_id = pool.intern_string(func_name);
            let func_id = pool.intern_function(crate::Function::Custom(name_id));

            let mut args = Vec::new();

            for arg_pair in inner {
                let arg_action = parse_action_pair(arg_pair, pool)?;
                args.push(arg_action);
            }

            if args.is_empty() {
                Ok(pool.add_action_with_location(
                    Action::Call {
                        fun: func_id,
                        last: 0,
                        arity: 0,
                    },
                    location,
                ))
            } else {
                let args_start = args.last().copied().unwrap();
                Ok(pool.add_action_with_location(
                    Action::Call {
                        fun: func_id,
                        last: pool.actions.len() - args_start.0,
                        arity: args.len(),
                    },
                    location,
                ))
            }
        }
        Rule::action_struct_expr => {
            let mut inner = pair.into_inner();
            let struct_name = inner.next().unwrap().as_str().to_string();
            let name_id = pool.intern_string(struct_name);

            let mut fields = Vec::new();

            for field_pair in inner {
                let field_action = parse_action_pair(field_pair, pool)?;
                fields.push(field_action);
            }

            if fields.is_empty() {
                Ok(pool.add_action_with_location(
                    Action::Struct {
                        name: name_id,
                        last: 0,
                        arity: 0,
                    },
                    location,
                ))
            } else {
                let fields_start = fields.last().copied().unwrap();
                Ok(pool.add_action_with_location(
                    Action::Struct {
                        name: name_id,
                        last: pool.actions.len() - fields_start.0,
                        arity: fields.len(),
                    },
                    location,
                ))
            }
        }
        Rule::var_action_function_call => {
            let mut inner = pair.into_inner();
            let var_name = inner.next().unwrap().as_str()[1..].to_string();
            let var_id = pool.intern_string(var_name);

            let mut args = Vec::new();

            for arg_pair in inner {
                let arg_action = parse_action_pair(arg_pair, pool)?;
                args.push(arg_action);
            }

            if args.is_empty() {
                Ok(pool.add_action_with_location(
                    Action::VarCallName {
                        var: var_id,
                        last: 0,
                        arity: 0,
                    },
                    location,
                ))
            } else {
                let args_start = args.last().copied().unwrap();
                Ok(pool.add_action_with_location(
                    Action::VarCallName {
                        var: var_id,
                        last: pool.actions.len() - args_start.0,
                        arity: args.len(),
                    },
                    location,
                ))
            }
        }
        Rule::var_action_struct_expr => {
            let mut inner = pair.into_inner();
            let var_name = inner.next().unwrap().as_str()[1..].to_string();
            let var_id = pool.intern_string(var_name);

            let mut fields = Vec::new();

            for field_pair in inner {
                let field_action = parse_action_pair(field_pair, pool)?;
                fields.push(field_action);
            }

            if fields.is_empty() {
                Ok(pool.add_action_with_location(
                    Action::VarStructName {
                        var: var_id,
                        last: 0,
                        arity: 0,
                    },
                    location,
                ))
            } else {
                let fields_start = fields.last().copied().unwrap();
                Ok(pool.add_action_with_location(
                    Action::VarStructName {
                        var: var_id,
                        last: pool.actions.len() - fields_start.0,
                        arity: fields.len(),
                    },
                    location,
                ))
            }
        }
        Rule::compute_expr => {
            let mut inner = pair.into_inner();
            let compute_inner = inner.next().unwrap();
            parse_compute_expr(compute_inner.as_str(), pool)
        }
        _ => Err(format!("Unexpected action rule: {:?}", pair.as_rule())),
    }
}
