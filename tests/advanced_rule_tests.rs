mod common;

use common::*;
use expression_explorer::rules::*;
use expression_explorer::*;
use std::collections::HashMap;

#[cfg(test)]
mod algebraic_transformations {
    use super::*;

    #[test]
    fn test_distributive_law_forward() {
        let (mut pool, expr) = parse_test_expr("(x * (y + z))");
        let pattern = parse_test_pattern_into("?a * (?b + ?c)", &mut pool);
        let action = parse_test_action_into("(a * b) + (a * c)", &mut pool);

        let rule_name = pool.intern_string("distribute_forward".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);

        let match_obj = Match {
            root: expr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj).unwrap();
        assert_expr_display(&pool, result, "((x * y) + (x * z))");
    }

    #[test]
    fn test_distributive_law_reverse() {
        let (mut pool, expr) = parse_test_expr("((x * y) + (x * z))");
        let pattern = parse_test_pattern_into("(?a * ?b) + (?a * ?c)", &mut pool);
        let action = parse_test_action_into("a * (b + c)", &mut pool);

        let rule_name = pool.intern_string("factor_out".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);

        let match_obj = Match {
            root: expr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj).unwrap();
        assert_expr_display(&pool, result, "(x * (y + z))");
    }

    #[test]
    fn test_associative_property() {
        let (mut pool, expr) = parse_test_expr("((x + y) + z)");
        let pattern = parse_test_pattern_into("(?a + ?b) + ?c", &mut pool);
        let action = parse_test_action_into("a + (b + c)", &mut pool);

        let rule_name = pool.intern_string("associative_add".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);

        let match_obj = Match {
            root: expr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj).unwrap();
        assert_expr_display(&pool, result, "(x + (y + z))");
    }

    #[test]
    fn test_power_rules() {
        let (mut pool, expr) = parse_test_expr("(x ^ 1)");
        let pattern = parse_test_pattern_into("?x ^ 1", &mut pool);
        let action = parse_test_action_into("x", &mut pool);

        let rule_name = pool.intern_string("power_identity".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);

        let match_obj = Match {
            root: expr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj).unwrap();
        assert_expr_display(&pool, result, "x");
    }
}

#[cfg(test)]
mod function_transformations {
    use super::*;

    #[test]
    fn test_function_simplification() {
        let (mut pool, expr) = parse_test_expr("sin(0)");
        let pattern = parse_test_pattern_into("sin(0)", &mut pool);
        let action = parse_test_action_into("0", &mut pool);

        let rule_name = pool.intern_string("sin_zero".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);

        let match_obj = Match {
            root: expr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj).unwrap();
        assert_expr_display(&pool, result, "0");
    }

    #[test]
    fn test_logarithm_properties() {
        let (mut pool, expr) = parse_test_expr("log((x * y))");
        let pattern = parse_test_pattern_into("log(?a * ?b)", &mut pool);
        let action = parse_test_action_into("(log(a) + log(b))", &mut pool);

        let rule_name = pool.intern_string("log_product".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);

        let match_obj = Match {
            root: expr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj).unwrap();
        assert_expr_display(&pool, result, "(log(x) + log(y))");
    }

    #[test]
    fn test_simple_function_matching() {
        let (mut pool, expr) = parse_test_expr("derivative((x + y))");
        let pattern = parse_test_pattern_into("derivative(?x + ?y)", &mut pool);
        let action = parse_test_action_into("(derivative(x) + derivative(y))", &mut pool);

        let rule_name = pool.intern_string("linear_derivative".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);

        let match_obj = Match {
            root: expr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj).unwrap();
        assert_expr_display(&pool, result, "(derivative(x) + derivative(y))");
    }
}

#[cfg(test)]
mod complex_pattern_scenarios {
    use super::*;

    #[test]
    fn test_nested_substitution() {
        let (mut pool, expr) = parse_test_expr("(((x + 0) * 1) + ((y + 0) * 1))");

        let pattern1 = parse_test_pattern_into("?x + 0", &mut pool);
        let action1 = parse_test_action_into("x", &mut pool);
        let rule1_name = pool.intern_string("identity_add".to_string());
        let rule1 = Rule {
            name: rule1_name,
            pattern: pattern1,
            action: action1,
        };
        let rule1_id = pool.add_rule(rule1);

        let pattern2 = parse_test_pattern_into("?x * 1", &mut pool);
        let action2 = parse_test_action_into("x", &mut pool);
        let rule2_name = pool.intern_string("identity_mult".to_string());
        let rule2 = Rule {
            name: rule2_name,
            pattern: pattern2,
            action: action2,
        };
        let rule2_id = pool.add_rule(rule2);

        let mut current_expr = expr;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                panic!("Too many iterations");
            }

            let matches = pool.find_matches(current_expr);
            if matches.is_empty() {
                break;
            }

            if let Some(new_expr) = pool.apply_rule(&matches[0]) {
                current_expr = new_expr;
            } else {
                break;
            }
        }

        let final_display = pool.display_with_children(current_expr);
        assert!(final_display.contains("x") && !final_display.contains("+ 0"));
    }

    #[test]
    fn test_conditional_rewriting() {
        let (mut pool, expr) = parse_test_expr("(a + a)");

        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);
        let action = parse_test_action_into("2 * x", &mut pool);
        let rule_name = pool.intern_string("double_add".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);

        let match_obj = Match {
            root: expr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj).unwrap();
        assert_expr_display(&pool, result, "(2 * a)");
    }

    #[test]
    fn test_multiple_variable_capture() {
        let (mut pool, expr) = parse_test_expr("(a + b)");
        let pattern = parse_test_pattern_into("?x + ?y", &mut pool);
        let action = parse_test_action_into("y + x", &mut pool);

        let rule_name = pool.intern_string("commutative".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);
        assert_eq!(captures.len(), 2);

        let match_obj = Match {
            root: expr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj).unwrap();
        assert_expr_display(&pool, result, "(b + a)");
    }
}

#[cfg(test)]
mod optimization_transformations {
    use super::*;

    #[test]
    fn test_constant_folding() {
        let (mut pool, expr) = parse_test_expr("((2 + 3) * (4 + 1))");

        let pattern = parse_test_pattern_into("2 + 3", &mut pool);
        let action = parse_test_action_into("5", &mut pool);
        let rule1_name = pool.intern_string("fold_2_3".to_string());
        let rule1 = Rule {
            name: rule1_name,
            pattern,
            action,
        };
        let rule1_id = pool.add_rule(rule1);

        let pattern2 = parse_test_pattern_into("4 + 1", &mut pool);
        let action2 = parse_test_action_into("5", &mut pool);
        let rule2_name = pool.intern_string("fold_4_1".to_string());
        let rule2 = Rule {
            name: rule2_name,
            pattern: pattern2,
            action: action2,
        };
        let rule2_id = pool.add_rule(rule2);

        let pattern3 = parse_test_pattern_into("5 * 5", &mut pool);
        let action3 = parse_test_action_into("25", &mut pool);
        let rule3_name = pool.intern_string("fold_5_5".to_string());
        let rule3 = Rule {
            name: rule3_name,
            pattern: pattern3,
            action: action3,
        };
        let _rule3_id = pool.add_rule(rule3);

        let mut current_expr = expr;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 5;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                break;
            }

            let matches = pool.find_matches(current_expr);
            if matches.is_empty() {
                break;
            }

            if let Some(new_expr) = pool.apply_rule(&matches[0]) {
                current_expr = new_expr;
            } else {
                break;
            }
        }

        let final_display = pool.display_with_children(current_expr);
        assert!(final_display.contains("5") || final_display == "25");
    }

    #[test]
    fn test_strength_reduction() {
        let (mut pool, expr) = parse_test_expr("(x * 2)");
        let pattern = parse_test_pattern_into("?x * 2", &mut pool);
        let action = parse_test_action_into("x + x", &mut pool);

        let rule_name = pool.intern_string("mult_by_2".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);

        let match_obj = Match {
            root: expr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj).unwrap();
        assert_expr_display(&pool, result, "(x + x)");
    }

    #[test]
    fn test_dead_code_elimination() {
        let (mut pool, expr) = parse_test_expr("((x + 0) * 1)");

        let pattern1 = parse_test_pattern_into("?x + 0", &mut pool);
        let action1 = parse_test_action_into("x", &mut pool);
        let rule1_name = pool.intern_string("elim_add_zero".to_string());
        let rule1 = Rule {
            name: rule1_name,
            pattern: pattern1,
            action: action1,
        };
        let rule1_id = pool.add_rule(rule1);

        let pattern2 = parse_test_pattern_into("?x * 1", &mut pool);
        let action2 = parse_test_action_into("x", &mut pool);
        let rule2_name = pool.intern_string("elim_mult_one".to_string());
        let rule2 = Rule {
            name: rule2_name,
            pattern: pattern2,
            action: action2,
        };
        let rule2_id = pool.add_rule(rule2);

        let mut current_expr = expr;
        loop {
            let matches = pool.find_matches(current_expr);
            if matches.is_empty() {
                break;
            }

            if let Some(new_expr) = pool.apply_rule(&matches[0]) {
                current_expr = new_expr;
            } else {
                break;
            }
        }

        assert_expr_display(&pool, current_expr, "x");
    }
}

#[cfg(test)]
mod rule_interaction_tests {
    use super::*;

    #[test]
    fn test_rule_priority_and_ordering() {
        let (mut pool, expr) = parse_test_expr("(x * 0)");

        let pattern1 = parse_test_pattern_into("?x * 0", &mut pool);
        let action1 = parse_test_action_into("0", &mut pool);
        let rule1_name = pool.intern_string("zero_mult".to_string());
        let rule1 = Rule {
            name: rule1_name,
            pattern: pattern1,
            action: action1,
        };
        let rule1_id = pool.add_rule(rule1);

        let pattern2 = parse_test_pattern_into("?x * ?y", &mut pool);
        let action2 = parse_test_action_into("y * x", &mut pool);
        let rule2_name = pool.intern_string("commutative".to_string());
        let rule2 = Rule {
            name: rule2_name,
            pattern: pattern2,
            action: action2,
        };
        let rule2_id = pool.add_rule(rule2);

        let matches = pool.find_matches(expr);

        let matching_rules: std::collections::HashSet<_> =
            matches.iter().map(|m| m.rule_id).collect();
        assert!(matching_rules.contains(&rule1_id));
        assert!(matching_rules.contains(&rule2_id));

        if let Some(result) = pool.apply_rule(&matches[0]) {
            let result_display = pool.display_with_children(result);
            assert!(result_display == "0" || result_display == "(0 * x)");
        }
    }

    #[test]
    fn test_recursive_rule_application() {
        let (mut pool, expr) = parse_test_expr("(((x + 0) + 0) + 0)");

        let pattern = parse_test_pattern_into("?x + 0", &mut pool);
        let action = parse_test_action_into("x", &mut pool);
        let rule_name = pool.intern_string("identity_add".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut current_expr = expr;
        let mut application_count = 0;

        loop {
            let matches = pool.find_matches(current_expr);
            let applicable_matches: Vec<_> =
                matches.iter().filter(|m| m.rule_id == rule_id).collect();

            if applicable_matches.is_empty() {
                break;
            }

            if let Some(new_expr) = pool.apply_rule(applicable_matches[0]) {
                current_expr = new_expr;
                application_count += 1;
            } else {
                break;
            }
        }

        assert_eq!(application_count, 3);
        assert_expr_display(&pool, current_expr, "x");
    }

    #[test]
    fn test_conflicting_rules() {
        let (mut pool, expr) = parse_test_expr("(a + b)");

        let pattern1 = parse_test_pattern_into("?x + ?y", &mut pool);
        let action1 = parse_test_action_into("y + x", &mut pool);
        let rule1_name = pool.intern_string("commutative".to_string());
        let rule1 = Rule {
            name: rule1_name,
            pattern: pattern1,
            action: action1,
        };
        let rule1_id = pool.add_rule(rule1);

        let pattern2 = parse_test_pattern_into("?x + ?y", &mut pool);
        let action2 = parse_test_action_into("x + y", &mut pool);
        let rule2_name = pool.intern_string("identity".to_string());
        let rule2 = Rule {
            name: rule2_name,
            pattern: pattern2,
            action: action2,
        };
        let rule2_id = pool.add_rule(rule2);

        let matches = pool.find_matches(expr);
        assert!(matches.len() >= 2);

        let matching_rules: std::collections::HashSet<_> =
            matches.iter().map(|m| m.rule_id).collect();
        assert!(matching_rules.contains(&rule1_id));
        assert!(matching_rules.contains(&rule2_id));
    }
}
