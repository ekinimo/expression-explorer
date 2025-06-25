mod common;

use common::*;
use expression_explorer::parser::patterns::parse_pattern;
use expression_explorer::parser::{parse_expression, parse_ruleset};

#[cfg(test)]
mod expression_parser_tests {
    use super::*;

    #[test]
    fn test_simple_variables() {
        let (pool, expr) = parse_test_expr("x");
        assert_expr_display(&pool, expr, "x");
        assert_eq!(count_nodes(&pool, expr), 1);
    }

    #[test]
    fn test_simple_numbers() {
        let (pool, expr) = parse_test_expr("42");
        assert_expr_display(&pool, expr, "42");
        assert_eq!(count_nodes(&pool, expr), 1);
    }

    #[test]
    fn test_simple_addition() {
        let (pool, expr) = parse_test_expr("(x + y)");
        assert_expr_display(&pool, expr, "(x + y)");
        assert_eq!(count_nodes(&pool, expr), 3);

        let children = get_children_vec(&pool, expr);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_simple_multiplication() {
        let (pool, expr) = parse_test_expr("(x * y)");
        assert_expr_display(&pool, expr, "(x * y)");
        assert_eq!(count_nodes(&pool, expr), 3);
    }

    #[test]
    fn test_operator_precedence() {
        let (pool, expr) = parse_test_expr("x + y * z");
        assert_expr_display(&pool, expr, "(x + (y * z))");

        let children = get_children_vec(&pool, expr);
        assert_eq!(children.len(), 2);

        let right_child = children[0];
        assert_expr_display(&pool, right_child, "(y * z)");
    }

    #[test]
    fn test_parentheses_override_precedence() {
        let (pool, expr) = parse_test_expr("(x + y) * z");
        assert_expr_display(&pool, expr, "((x + y) * z)");

        let children = get_children_vec(&pool, expr);
        assert_eq!(children.len(), 2);

        let left_child = children[1];
        assert_expr_display(&pool, left_child, "(x + y)");
    }

    #[test]
    fn test_nested_expressions() {
        let (pool, expr) = parse_test_expr("((a + b) + (c + d))");
        assert_expr_display(&pool, expr, "((a + b) + (c + d))");

        let children = get_children_vec(&pool, expr);
        assert_eq!(children.len(), 2);

        let left_child = children[1];
        let right_child = children[0];

        assert_expr_display(&pool, left_child, "(a + b)");
        assert_expr_display(&pool, right_child, "(c + d)");
    }

    #[test]
    fn test_deeply_nested() {
        let (pool, expr) = parse_test_expr("(((x + y) + z) + w)");
        assert_expr_display(&pool, expr, "(((x + y) + z) + w)");

        let children = get_children_vec(&pool, expr);
        assert_eq!(children.len(), 2);

        let left_child = children[1];
        assert_expr_display(&pool, left_child, "((x + y) + z)");
    }

    #[test]
    fn test_repeated_variables() {
        let (pool, expr) = parse_test_expr("(x + x)");
        assert_expr_display(&pool, expr, "(x + x)");
        assert_eq!(count_nodes(&pool, expr), 3);
    }

    #[test]
    fn test_complex_repeated_structure() {
        let (pool, expr) = parse_test_expr("((x+x)+(x+x)) + ((x+x)+(x+x))");
        assert_expr_display(&pool, expr, "(((x + x) + (x + x)) + ((x + x) + (x + x)))");

        let total_nodes = pool.exprs.len();

        assert!(total_nodes < 50, "Too many nodes: {}", total_nodes);
    }

    #[test]
    fn test_function_calls() {
        let (pool, expr) = parse_test_expr("sin(x)");
        assert_expr_display(&pool, expr, "sin(x)");

        let children = get_children_vec(&pool, expr);
        assert_eq!(children.len(), 1);
        assert_expr_display(&pool, children[0], "x");
    }

    #[test]
    fn test_function_calls_multiple_args() {
        let (pool, expr) = parse_test_expr("pow(x, y)");
        assert_expr_display(&pool, expr, "pow(x, y)");

        let children = get_children_vec(&pool, expr);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_power_operator() {
        let (pool, expr) = parse_test_expr("x ^ y");
        assert_expr_display(&pool, expr, "(x ^ y)");
    }

    #[test]
    fn test_unary_operators() {
        let (pool, expr) = parse_test_expr("-x");
        assert_expr_display(&pool, expr, "(-x)");

        let children = get_children_vec(&pool, expr);
        assert_eq!(children.len(), 1);
        assert_expr_display(&pool, children[0], "x");
    }
}

#[cfg(test)]
mod pattern_parser_tests {
    use super::*;

    #[test]
    fn test_simple_wildcard() {
        let (pool, pattern) = parse_test_pattern("?x");
        assert!(pattern.0 < pool.patterns.len());
    }

    #[test]
    fn test_pattern_with_structure() {
        let (pool, pattern) = parse_test_pattern("?x + ?y");
        assert!(pattern.0 < pool.patterns.len());
    }

    #[test]
    fn test_repeated_pattern_variable() {
        let (pool, pattern) = parse_test_pattern("?x + ?x");
        assert!(pattern.0 < pool.patterns.len());
    }

    #[test]
    fn test_pattern_with_constants() {
        let (pool, pattern) = parse_test_pattern("?x + 0");
        assert!(pattern.0 < pool.patterns.len());
    }

    #[test]
    fn test_pattern_with_functions() {
        let (pool, pattern) = parse_test_pattern("sin(?x)");
        assert!(pattern.0 < pool.patterns.len());
    }

    #[test]
    fn test_pattern_with_parentheses() {
        let (pool, pattern) = parse_test_pattern("?x + (?y + ?z)");
        assert!(pattern.0 < pool.patterns.len());
    }

    #[test]
    fn test_ruleset_with_parentheses_pattern() {
        let ruleset_text = r#"algebra {
  double_add: ?x + ?x => 2 * x
  identity_add: ?x + 0 => x
  add_identity: 0 + ?x => x
  assoc: ?x + (?y + ?z) => (x+y)+z
}"#;
        let (pool, ruleset) = parse_test_ruleset(ruleset_text);
        let ruleset_data = &pool[ruleset];
        println!(
            "Expected 4 rules, found {}",
            ruleset_data.rules_end - ruleset_data.rules_start
        );

        // Print all rules for debugging
        for i in ruleset_data.rules_start..ruleset_data.rules_end {
            let rule = &pool.rules[i];
            let rule_name = pool.display_name(rule.name);
            println!("Rule {}: {}", i - ruleset_data.rules_start, rule_name);
        }

        assert_eq!(ruleset_data.rules_end - ruleset_data.rules_start, 4);
        assert_eq!(pool.get_ruleset_rule_count(ruleset), 4);
    }

    #[test]
    fn test_monoid_ruleset_count() {
        let ruleset_text = r#"monoid {
  left_identity  : 0 + ?x         => x
  right_identity : ?x + 0         => x
  associativity  : (?x + ?y) + ?z => x + (y + z)
}"#;
        let (pool, ruleset) = parse_test_ruleset(ruleset_text);
        let rule_count = pool.get_ruleset_rule_count(ruleset);
        println!("Monoid ruleset has {} rules", rule_count);

        // Print all rules for debugging
        let ruleset_data = &pool[ruleset];
        for i in ruleset_data.rules_start..ruleset_data.rules_end {
            let rule = &pool.rules[i];
            let rule_name = pool.display_name(rule.name);
            println!("Rule {}: {}", i - ruleset_data.rules_start, rule_name);
        }

        assert_eq!(rule_count, 3);
    }
}

#[cfg(test)]
mod action_parser_tests {
    use super::*;

    #[test]
    fn test_simple_variable_action() {
        let (pool, action) = parse_test_action("x");
        assert!(action.0 < pool.actions.len());
    }

    #[test]
    fn test_simple_number_action() {
        let (pool, action) = parse_test_action("42");
        assert!(action.0 < pool.actions.len());
    }

    #[test]
    fn test_computation_action() {
        let (pool, action) = parse_test_action("2 * x");
        assert!(action.0 < pool.actions.len());
    }

    #[test]
    fn test_complex_action() {
        let (pool, action) = parse_test_action("(x + y) * 2");
        assert!(action.0 < pool.actions.len());
    }
}

#[cfg(test)]
mod ruleset_parser_tests {
    use super::*;

    #[test]
    fn test_simple_rule() {
        let ruleset_text = "test_rules {\n  double: ?x + ?x => 2 * x\n}";
        let (pool, ruleset) = parse_test_ruleset(ruleset_text);

        let ruleset_data = &pool[ruleset];
        assert_eq!(ruleset_data.rules_end - ruleset_data.rules_start, 1);
    }

    #[test]
    fn test_multiple_rules() {
        let ruleset_text = r#"algebra_rules {
  double_add: ?x + ?x => 2 * x
  zero_mult: ?x * 0 => 0
  identity_add: ?x + 0 => x
}"#;
        let (pool, ruleset) = parse_test_ruleset(ruleset_text);

        let ruleset_data = &pool[ruleset];
        assert_eq!(ruleset_data.rules_end - ruleset_data.rules_start, 3);
    }

    #[test]
    fn test_complex_rules() {
        let ruleset_text = r#"complex_rules {
  distribute: ?x * ?y => x * y
  simple_add: ?x + ?y => y + x
}"#;
        let (pool, ruleset) = parse_test_ruleset(ruleset_text);

        let ruleset_data = &pool[ruleset];
        assert_eq!(ruleset_data.rules_end - ruleset_data.rules_start, 2);
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_expression_syntax() {
        let mut pool = new_test_pool();
        let result = parse_expression("x +", &mut pool);
        assert!(result.is_err());
    }

    #[test]
    fn test_mismatched_parentheses() {
        let mut pool = new_test_pool();
        let result = parse_expression("(x + y", &mut pool);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_pattern_syntax() {
        let mut pool = new_test_pool();
        let result = parse_pattern("?", &mut pool);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_ruleset_syntax() {
        let mut pool = new_test_pool();
        let result = parse_ruleset("invalid ruleset", &mut pool);
        assert!(result.is_err());
    }
}
