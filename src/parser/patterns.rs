use crate::{Function, Pattern, PatternId, Pool};
use pest::Parser;
use pest::iterators::Pair;

use crate::parser::pattern_parser::{PatternParser, Rule};

pub fn parse_pattern(input: &str, pool: &mut Pool) -> Result<PatternId, String> {
    let pairs =
        PatternParser::parse(Rule::pattern, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs.into_iter().next().unwrap();
    parse_pattern_pair(pair, pool)
}

pub fn parse_pattern_pair(pair: Pair<Rule>, pool: &mut Pool) -> Result<PatternId, String> {
    match pair.as_rule() {
        Rule::pattern => parse_pattern_pair(pair.into_inner().next().unwrap(), pool),
        Rule::pattern_sum => {
            let mut inner = pair.into_inner();
            let mut pattern = parse_pattern_pair(inner.next().unwrap(), pool)?;

            while let Some(op) = inner.next() {
                let right = parse_pattern_pair(inner.next().unwrap(), pool)?;
                let func = match op.as_str() {
                    "+" => Function::Add,
                    "-" => Function::Subtract,
                    _ => return Err(format!("Unknown operator: {}", op.as_str())),
                };
                let fun_id = pool.intern_function(func);

                let new_node_pos = pool.patterns.len();
                let first_child_pos = if pattern.0 < right.0 {
                    pattern.0
                } else {
                    right.0
                };
                let last = new_node_pos - first_child_pos;

                pattern = pool.add_pattern(Pattern::Call {
                    fun: fun_id,
                    last,
                    arity: 2,
                });
            }
            Ok(pattern)
        }
        Rule::pattern_product => {
            let mut inner = pair.into_inner();
            let mut pattern = parse_pattern_pair(inner.next().unwrap(), pool)?;

            while let Some(op) = inner.next() {
                let right = parse_pattern_pair(inner.next().unwrap(), pool)?;
                let func = match op.as_str() {
                    "*" => Function::Multiply,
                    "/" => Function::Divide,
                    _ => return Err(format!("Unknown operator: {}", op.as_str())),
                };
                let fun_id = pool.intern_function(func);

                let new_node_pos = pool.patterns.len();
                let first_child_pos = if pattern.0 < right.0 {
                    pattern.0
                } else {
                    right.0
                };
                let last = new_node_pos - first_child_pos;

                pattern = pool.add_pattern(Pattern::Call {
                    fun: fun_id,
                    last,
                    arity: 2,
                });
            }
            Ok(pattern)
        }
        Rule::pattern_power => {
            let mut inner = pair.into_inner();
            let mut pattern = parse_pattern_pair(inner.next().unwrap(), pool)?;

            for right_pair in inner {
                let right = parse_pattern_pair(right_pair, pool)?;
                let fun_id = pool.intern_function(Function::Power);

                let new_node_pos = pool.patterns.len();
                let first_child_pos = if pattern.0 < right.0 {
                    pattern.0
                } else {
                    right.0
                };
                let last = new_node_pos - first_child_pos;

                pattern = pool.add_pattern(Pattern::Call {
                    fun: fun_id,
                    last,
                    arity: 2,
                });
            }
            Ok(pattern)
        }
        Rule::pattern_value => {
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
            let mut pattern = parse_pattern_pair(atom_pair.unwrap(), pool)?;

            for func in ops.into_iter().rev() {
                let fun_id = pool.intern_function(func);

                pattern = pool.add_pattern(Pattern::Call {
                    fun: fun_id,
                    last: pool.patterns.len() - pattern.0,
                    arity: 1,
                });
            }

            Ok(pattern)
        }
        Rule::any_number => {
            let name = &pair.as_str()[1..];
            let var_id = pool.intern_string(name.to_string());
            Ok(pool.add_pattern(Pattern::AnyNumber(var_id)))
        }
        Rule::named_variable => {
            let name = &pair.as_str()[1..];
            let var_id = pool.intern_string(name.to_string());
            Ok(pool.add_pattern(Pattern::Wildcard(var_id)))
        }
        Rule::number => {
            let num = pair
                .as_str()
                .parse::<i32>()
                .map_err(|_| format!("Invalid number: {}", pair.as_str()))?;
            Ok(pool.add_pattern(Pattern::Number(num)))
        }
        Rule::variable => {
            let var_name = pair.as_str().to_string();
            let var_id = pool.intern_string(var_name);
            Ok(pool.add_pattern(Pattern::Variable(var_id)))
        }
        Rule::pattern_function_call => {
            let mut inner = pair.into_inner();
            let func_name = inner.next().unwrap().as_str().to_string();
            let name_id = pool.intern_string(func_name);
            let func_id = pool.intern_function(crate::Function::Custom(name_id));

            let mut args = Vec::new();

            for arg_pair in inner {
                let arg_pattern = parse_pattern_pair(arg_pair, pool)?;
                args.push(arg_pattern);
            }

            let args_start = args.last().copied().unwrap();

            Ok(pool.add_pattern(Pattern::Call {
                fun: func_id,
                last: pool.patterns.len() - args_start.0,
                arity: args.len(),
            }))
        }
        Rule::pattern_struct_expr => {
            let mut inner = pair.into_inner();
            let struct_name = inner.next().unwrap().as_str().to_string();
            let name_id = pool.intern_string(struct_name);

            let mut fields = Vec::new();

            for field_pair in inner {
                let field_pattern = parse_pattern_pair(field_pair, pool)?;
                fields.push(field_pattern);
            }

            let fields_start = fields.last().copied().unwrap();

            Ok(pool.add_pattern(Pattern::Struct {
                name: name_id,
                last: pool.patterns.len() - fields_start.0,
                arity: fields.len(),
            }))
        }
        Rule::var_function_call => {
            let mut inner = pair.into_inner();
            let var_name = inner.next().unwrap().as_str()[1..].to_string();
            let var_id = pool.intern_string(var_name);

            let mut args = Vec::new();

            for arg_pair in inner {
                let arg_pattern = parse_pattern_pair(arg_pair, pool)?;
                args.push(arg_pattern);
            }

            let args_start = args.last().copied().unwrap();

            Ok(pool.add_pattern(Pattern::VarCallName {
                var: var_id,
                last: pool.patterns.len() - args_start.0,
                arity: args.len(),
            }))
        }
        Rule::var_struct_expr => {
            let mut inner = pair.into_inner();
            let var_name = inner.next().unwrap().as_str()[1..].to_string();
            let var_id = pool.intern_string(var_name);

            let mut fields = Vec::new();

            for field_pair in inner {
                let field_pattern = parse_pattern_pair(field_pair, pool)?;
                fields.push(field_pattern);
            }

            let fields_start = fields.last().copied().unwrap();

            Ok(pool.add_pattern(Pattern::VarStructName {
                var: var_id,
                last: pool.patterns.len() - fields_start.0,
                arity: fields.len(),
            }))
        }
        _ => Err(format!("Unexpected pattern rule: {:?}", pair.as_rule())),
    }
}
