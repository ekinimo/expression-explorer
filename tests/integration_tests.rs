mod common;

use common::*;
use expression_explorer::children::Children;
use expression_explorer::parser::parse_expression;
use expression_explorer::*;

#[cfg(test)]
mod end_to_end_workflows {
    use super::*;

    #[test]
    fn test_simple_algebraic_simplification() {
        let (mut pool, expr) = parse_test_expr("((x + 0) + (x + x))");

        let pattern1 = parse_test_pattern_into("?x + 0", &mut pool);
        let action1 = parse_test_action_into("x", &mut pool);
        let rule1_name = pool.intern_string("identity_add".to_string());
        let rule1 = Rule {
            name: rule1_name,
            pattern: pattern1,
            action: action1,
        };
        let _rule1_id = pool.add_rule(rule1);

        let pattern2 = parse_test_pattern_into("?x + ?x", &mut pool);
        let action2 = parse_test_action_into("2 * x", &mut pool);
        let rule2_name = pool.intern_string("double_add".to_string());
        let rule2 = Rule {
            name: rule2_name,
            pattern: pattern2,
            action: action2,
        };
        let _rule2_id = pool.add_rule(rule2);

        let matches = pool.find_matches(expr);

        assert!(matches.len() >= 2);

        let first_match = &matches[0];
        let result1 = pool.apply_rule(first_match);
        assert!(result1.is_some());

        let new_expr = result1.unwrap();
        let display = pool.display_with_children(new_expr);

        assert!(
            display.len() < pool.display_with_children(expr).len()
                || display.contains("2 *")
                || !display.contains("+ 0")
        );
    }

    #[test]
    fn test_multiple_rule_applications() {
        let (mut pool, mut expr) = parse_test_expr("(((x + 0) + (x + 0)) + 0)");

        let pattern = parse_test_pattern_into("?x + 0", &mut pool);
        let action = parse_test_action_into("x", &mut pool);
        let rule_name = pool.intern_string("identity_add".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                panic!("Too many iterations, possible infinite loop");
            }

            let matches = pool.find_matches(expr);

            let applicable_match = matches.iter().find(|m| m.rule_id == rule_id);

            if let Some(match_obj) = applicable_match {
                if let Some(new_expr) = pool.apply_rule(match_obj) {
                    expr = new_expr;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let final_display = pool.display_with_children(expr);
        assert!(!final_display.contains("+ 0"));
        assert!(final_display.contains("x"));
    }

    #[test]
    fn test_nested_expression_transformation() {
        let (mut pool, expr) = parse_test_expr("(((x + x) + (y + y)) + ((a + a) + (b + b)))");

        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);
        let action = parse_test_action_into("2 * x", &mut pool);
        let rule_name = pool.intern_string("double_add".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let matches = pool.find_matches(expr);

        let double_matches: Vec<_> = matches.iter().filter(|m| m.rule_id == rule_id).collect();

        assert!(double_matches.len() >= 4);

        if let Some(result) = pool.apply_rule(double_matches[0]) {
            let result_display = pool.display_with_children(result);
            assert!(result_display.contains("2 *"));
        }
    }
}

#[cfg(test)]
mod ruleset_integration {
    use super::*;

    #[test]
    fn test_complete_ruleset_workflow() {
        let ruleset_text = r#"basic_algebra {
  identity_add: ?x + 0 => x
  identity_mult: ?x * 1 => x
  zero_mult: ?x * 0 => 0
  double_add: ?x + ?x => 2 * x
}"#;

        let (mut pool, ruleset_id) = parse_test_ruleset(ruleset_text);

        let ruleset = &pool[ruleset_id];
        let rules_count = ruleset.rules_end - ruleset.rules_start;
        assert_eq!(rules_count, 4);

        let expr_text = "((x + 0) * (y + y))";
        let expr = parse_expression(expr_text, &mut pool).expect("Failed to parse expression");

        let matches = pool.find_matches(expr);
        assert!(matches.len() > 0);

        if let Some(result) = pool.apply_rule(&matches[0]) {
            let result_display = pool.display_with_children(result);
            assert_ne!(result_display, pool.display_with_children(expr));
        }
    }

    #[test]
    fn test_commutative_transformations() {
        let (mut pool, expr) = parse_test_expr("(a + b)");

        let pattern = parse_test_pattern_into("?x + ?y", &mut pool);
        let action = parse_test_action_into("y + x", &mut pool);
        let rule_name = pool.intern_string("commutative_add".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let matches = pool.find_matches(expr);
        let comm_match = matches.iter().find(|m| m.rule_id == rule_id).unwrap();

        let result = pool.apply_rule(comm_match).unwrap();

        assert_expr_display(&pool, result, "(b + a)");
    }
}

#[cfg(test)]
mod performance_and_edge_cases {
    use super::*;

    #[test]
    fn test_large_expression_handling() {
        let expr_text = "((((x + x) + (x + x)) + ((x + x) + (x + x))) + (((x + x) + (x + x)) + ((x + x) + (x + x))))";
        let (mut pool, expr) = parse_test_expr(expr_text);

        assert!(pool.length(expr) > 20);

        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);
        let action = parse_test_action_into("2 * x", &mut pool);
        let rule_name = pool.intern_string("double_add".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let _rule_id = pool.add_rule(rule);

        let matches = pool.find_matches(expr);
        assert!(matches.len() > 5);
    }

    #[test]
    fn test_no_infinite_loops() {
        let (mut pool, expr) = parse_test_expr("(x + y)");

        let pattern = parse_test_pattern_into("?x + ?y", &mut pool);
        let action = parse_test_action_into("y + x", &mut pool);
        let rule_name = pool.intern_string("swap".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let matches1 = pool.find_matches(expr);
        let match1 = matches1.iter().find(|m| m.rule_id == rule_id).unwrap();
        let result1 = pool.apply_rule(match1).unwrap();

        let matches2 = pool.find_matches(result1);
        let match2 = matches2.iter().find(|m| m.rule_id == rule_id).unwrap();
        let result2 = pool.apply_rule(match2).unwrap();

        let original_display = pool.display_with_children(expr);
        let final_display = pool.display_with_children(result2);
        assert_eq!(original_display, final_display);
    }

    #[test]
    fn test_pattern_matching_edge_cases() {
        let (mut pool, expr) = parse_test_expr("x");

        let pattern = parse_test_pattern_into("?x + ?y", &mut pool);
        let action = parse_test_action_into("x + y", &mut pool);
        let rule_name = pool.intern_string("no_match".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let _rule_id = pool.add_rule(rule);

        let matches = pool.find_matches(expr);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_memory_efficiency() {
        let (pool, _expr) = parse_test_expr("((x + x) + (x + x))");

        let x_count = pool.names.iter().filter(|name| name == &"x").count();
        assert_eq!(x_count, 1);

        assert!(pool.exprs.len() < 20);
    }
}
