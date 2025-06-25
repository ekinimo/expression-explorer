mod common;

use common::*;
use expression_explorer::rules::*;
use expression_explorer::*;
use std::collections::HashMap;

#[cfg(test)]
mod simple_rule_application {
    use super::*;

    #[test]
    fn test_double_addition_rule() {
        let (mut pool, expr) = parse_test_expr("(x + x)");
        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);
        let action = parse_test_action_into("2 * x", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
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

        let result = pool.apply_rule(&match_obj);
        assert!(result.is_some());

        let new_expr = result.unwrap();
        assert_expr_display(&pool, new_expr, "(2 * x)");
    }

    #[test]
    fn test_zero_multiplication_rule() {
        let (mut pool, expr) = parse_test_expr("(x * 0)");
        let pattern = parse_test_pattern_into("?x * 0", &mut pool);
        let action = parse_test_action_into("0", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
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

        let result = pool.apply_rule(&match_obj);
        assert!(result.is_some());

        let new_expr = result.unwrap();
        assert_expr_display(&pool, new_expr, "0");
    }

    #[test]
    fn test_identity_addition_rule() {
        let (mut pool, expr) = parse_test_expr("(x + 0)");
        let pattern = parse_test_pattern_into("?x + 0", &mut pool);
        let action = parse_test_action_into("x", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
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

        let result = pool.apply_rule(&match_obj);
        assert!(result.is_some());

        let new_expr = result.unwrap();
        assert_expr_display(&pool, new_expr, "x");
    }

    #[test]
    fn test_numeric_computation() {
        let (mut pool, expr) = parse_test_expr("(2 + 3)");
        let pattern = parse_test_pattern_into("2 + 3", &mut pool);
        let action = parse_test_action_into("5", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
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

        let result = pool.apply_rule(&match_obj);
        assert!(result.is_some());

        let new_expr = result.unwrap();
        assert_expr_display(&pool, new_expr, "5");
    }
}

#[cfg(test)]
mod complex_rule_application {
    use super::*;

    #[test]
    fn test_distributive_law() {
        let (mut pool, expr) = parse_test_expr("(a * (b + c))");
        let pattern = parse_test_pattern_into("?x * (?y + ?z)", &mut pool);
        let action = parse_test_action_into("(x * y) + (x * z)", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
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

        let result = pool.apply_rule(&match_obj);
        assert!(result.is_some());

        let new_expr = result.unwrap();
        assert_expr_display(&pool, new_expr, "((a * b) + (a * c))");
    }

    #[test]
    fn test_commutative_property() {
        let (mut pool, expr) = parse_test_expr("(a + b)");
        let pattern = parse_test_pattern_into("?x + ?y", &mut pool);
        let action = parse_test_action_into("y + x", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
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

        let result = pool.apply_rule(&match_obj);
        assert!(result.is_some());

        let new_expr = result.unwrap();
        assert_expr_display(&pool, new_expr, "(b + a)");
    }

    #[test]
    fn test_nested_rule_application() {
        let (mut pool, expr) = parse_test_expr("((x + x) + y)");
        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);
        let action = parse_test_action_into("2 * x", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let children = get_children_vec(&pool, expr);
        let subexpr = children[1];

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, subexpr, &mut captures);
        assert!(matches);

        let match_obj = Match {
            root: subexpr,
            offset: expr,
            rule_id,
            captures,
        };

        let result = pool.apply_rule(&match_obj);
        assert!(result.is_some());

        let new_subexpr = result.unwrap();
        assert_expr_display(&pool, new_subexpr, "(2 * x)");
    }
}

#[cfg(test)]
mod rule_matching_tests {
    use super::*;

    #[test]
    fn test_find_matches_simple() {
        let (mut pool, expr) = parse_test_expr("(x + x)");
        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);
        let action = parse_test_action_into("2 * x", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let matches = pool.find_matches(expr);
        assert_eq!(matches.len(), 1);

        let match_obj = &matches[0];
        assert_eq!(match_obj.root, expr);
        assert_eq!(match_obj.rule_id, rule_id);
        assert_eq!(match_obj.captures.len(), 1);
    }

    #[test]
    fn test_find_matches_multiple() {
        let (mut pool, expr) = parse_test_expr("((x + x) + (y + y))");
        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);
        let action = parse_test_action_into("2 * x", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let _rule_id = pool.add_rule(rule);

        let matches = pool.find_matches(expr);
        assert!(matches.len() >= 2);
    }

    #[test]
    fn test_find_matches_no_match() {
        let (mut pool, expr) = parse_test_expr("(x * y)");
        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);
        let action = parse_test_action_into("2 * x", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
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
    fn test_multiple_rules() {
        let (mut pool, expr) = parse_test_expr("(x + 0)");

        let pattern1 = parse_test_pattern_into("?x + 0", &mut pool);
        let action1 = parse_test_action_into("x", &mut pool);
        let rule1_name = pool.intern_string("rule1".to_string());
        let rule1 = Rule {
            name: rule1_name,
            pattern: pattern1,
            action: action1,
        };
        let rule_id1 = pool.add_rule(rule1);

        let pattern2 = parse_test_pattern_into("?x + ?x", &mut pool);
        let action2 = parse_test_action_into("2 * x", &mut pool);
        let rule2_name = pool.intern_string("rule2".to_string());
        let rule2 = Rule {
            name: rule2_name,
            pattern: pattern2,
            action: action2,
        };
        let _rule_id2 = pool.add_rule(rule2);

        let matches = pool.find_matches(expr);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].rule_id, rule_id1);
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_rule_application_failure() {
        let (mut pool, expr) = parse_test_expr("(x + y)");
        let pattern = parse_test_pattern_into("?x + ?y", &mut pool);
        let action = parse_test_action_into("z", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
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

        let result = pool.apply_rule(&match_obj);
        assert!(result.is_some());
    }

    #[test]
    fn test_computation_with_non_numeric() {
        let (mut pool, expr) = parse_test_expr("(x + y)");
        let pattern = parse_test_pattern_into("?x + ?y", &mut pool);
        let action = parse_test_action_into("x + y", &mut pool);

        let rule_name = pool.intern_string("test_rule".to_string());
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

        let result = pool.apply_rule(&match_obj);
        assert!(result.is_some());

        let new_expr = result.unwrap();
        assert_expr_display(&pool, new_expr, "(x + y)");
    }
}
