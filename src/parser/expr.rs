use crate::children::Children;
use crate::{ExprId, ExprNode, Function, Location, Pool, Provenance};
use pest::Parser;
use pest::iterators::Pair;

use crate::parser::expr_parser::{ExprParser, Rule};

pub fn parse_expression(input: &str, pool: &mut Pool) -> Result<ExprId, String> {
    let pairs =
        ExprParser::parse(Rule::expression, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs.into_iter().next().unwrap();
    let expr_id = parse_expr_pair(pair, pool)?;

    pool.mark_expr_end(expr_id);

    Ok(expr_id)
}

pub fn parse_expr_pair(pair: Pair<Rule>, pool: &mut Pool) -> Result<ExprId, String> {
    let span = pair.as_span();
    let location = Location::new(span.start(), span.end());

    match pair.as_rule() {
        Rule::expression => parse_expr_pair(pair.into_inner().next().unwrap(), pool),
        Rule::sum => {
            let mut inner = pair.into_inner();
            let mut left = parse_expr_pair(inner.next().unwrap(), pool)?;

            while let Some(op) = inner.next() {
                let right = parse_expr_pair(inner.next().unwrap(), pool)?;
                let func = match op.as_str() {
                    "+" => Function::Add,
                    "-" => Function::Subtract,
                    _ => return Err(format!("Unknown operator: {}", op.as_str())),
                };
                let fun_id = pool.intern_function(func);

                let new_node_pos = pool.exprs.len();

                let left_start = left.0 + 1 - pool.length(left);
                let right_start = right.0 + 1 - pool.length(right);

                let first_child_pos = if left_start < right_start {
                    left_start
                } else {
                    right_start
                };
                let last = new_node_pos - first_child_pos;

                left = pool.add_expr_with_provenance(
                    ExprNode::Call {
                        fun: fun_id,
                        last,
                        arity: 2,
                    },
                    Provenance::Parsed(location.clone()),
                );
            }
            Ok(left)
        }
        Rule::product => {
            let mut inner = pair.into_inner();
            let mut left = parse_expr_pair(inner.next().unwrap(), pool)?;

            while let Some(op) = inner.next() {
                let right = parse_expr_pair(inner.next().unwrap(), pool)?;
                let func = match op.as_str() {
                    "*" => Function::Multiply,
                    "/" => Function::Divide,
                    _ => return Err(format!("Unknown operator: {}", op.as_str())),
                };
                let fun_id = pool.intern_function(func);

                let new_node_pos = pool.exprs.len();

                let left_start = left.0 + 1 - pool.length(left);
                let right_start = right.0 + 1 - pool.length(right);

                let first_child_pos = if left_start < right_start {
                    left_start
                } else {
                    right_start
                };
                let last = new_node_pos - first_child_pos;

                left = pool.add_expr_with_provenance(
                    ExprNode::Call {
                        fun: fun_id,
                        last,
                        arity: 2,
                    },
                    Provenance::Parsed(location.clone()),
                );
            }
            Ok(left)
        }
        Rule::power => {
            let mut inner = pair.into_inner();
            let mut left = parse_expr_pair(inner.next().unwrap(), pool)?;

            for right_pair in inner {
                let right = parse_expr_pair(right_pair, pool)?;
                let fun_id = pool.intern_function(Function::Power);

                let new_node_pos = pool.exprs.len();

                let left_start = left.0 + 1 - pool.length(left);
                let right_start = right.0 + 1 - pool.length(right);

                let first_child_pos = if left_start < right_start {
                    left_start
                } else {
                    right_start
                };
                let last = new_node_pos - first_child_pos;

                left = pool.add_expr_with_provenance(
                    ExprNode::Call {
                        fun: fun_id,
                        last,
                        arity: 2,
                    },
                    Provenance::Parsed(location.clone()),
                );
            }
            Ok(left)
        }
        Rule::value => {
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
            let mut expr = parse_expr_pair(atom_pair.unwrap(), pool)?;

            for func in ops.into_iter().rev() {
                let fun_id = pool.intern_function(func);

                expr = pool.add_expr_with_provenance(
                    ExprNode::Call {
                        fun: fun_id,
                        last: pool.exprs.len() - expr.0,
                        arity: 1,
                    },
                    Provenance::Parsed(location.clone()),
                );
            }

            Ok(expr)
        }
        Rule::number => {
            let num = pair
                .as_str()
                .parse::<i32>()
                .map_err(|_| format!("Invalid number: {}", pair.as_str()))?;
            Ok(pool.add_expr_with_provenance(ExprNode::Number(num), Provenance::Parsed(location)))
        }
        Rule::variable => {
            let var_name = pair.as_str().to_string();
            let var_id = pool.intern_string(var_name);
            Ok(pool
                .add_expr_with_provenance(ExprNode::Variable(var_id), Provenance::Parsed(location)))
        }
        Rule::function_call => {
            let mut inner = pair.into_inner();
            let func_name = inner.next().unwrap().as_str().to_string();
            let name_id = pool.intern_string(func_name);
            let func_id = pool.intern_function(crate::Function::Custom(name_id));

            let mut args = Vec::new();

            for arg_pair in inner {
                let arg_expr = parse_expr_pair(arg_pair, pool)?;
                args.push(arg_expr);
            }

            let last_offset = if args.is_empty() {
                0
            } else {
                pool.exprs.len() - args[0].0
            };

            Ok(pool.add_expr_with_provenance(
                ExprNode::Call {
                    fun: func_id,
                    last: last_offset,
                    arity: args.len(),
                },
                Provenance::Parsed(location),
            ))
        }
        Rule::struct_expr => {
            let mut inner = pair.into_inner();
            let struct_name = inner.next().unwrap().as_str().to_string();
            let name_id = pool.intern_string(struct_name);

            let mut fields = Vec::new();

            for field_pair in inner {
                let field_expr = parse_expr_pair(field_pair, pool)?;
                fields.push(field_expr);
            }

            let last_offset = if fields.is_empty() {
                0
            } else {
                pool.exprs.len() - fields[0].0
            };

            Ok(pool.add_expr_with_provenance(
                ExprNode::Struct {
                    name: name_id,
                    last: last_offset,
                    arity: fields.len(),
                },
                Provenance::Parsed(location),
            ))
        }
        _ => Err(format!("Unexpected rule: {:?}", pair.as_rule())),
    }
}
