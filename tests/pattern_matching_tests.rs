mod common;

use common::*;
use expression_explorer::rules;
use expression_explorer::*;
use std::collections::HashMap;

#[cfg(test)]
mod simple_pattern_matching {
    use super::*;

    #[test]
    fn test_simple_variable_match() {
        let (mut pool, expr) = parse_test_expr("x");
        let pattern = parse_test_pattern_into("?x", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 1);

        if let Some(rules::CapturedValue::Expression(captured_expr)) = captures.values().next() {
            assert_expr_display(&pool, *captured_expr, "x");
        } else {
            panic!("Expected captured expression");
        }
    }

    #[test]
    fn test_simple_number_match() {
        let (mut pool, expr) = parse_test_expr("42");
        let pattern = parse_test_pattern_into("42", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 0);
    }

    #[test]
    fn test_number_wildcard_match() {
        let (mut pool, expr) = parse_test_expr("42");
        let pattern = parse_test_pattern_into("?n", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 1);

        if let Some(rules::CapturedValue::Expression(captured_expr)) = captures.values().next() {
            assert_expr_display(&pool, *captured_expr, "42");
        } else {
            panic!("Expected captured expression");
        }
    }

    #[test]
    fn test_no_match_different_literals() {
        let (mut pool, expr) = parse_test_expr("42");
        let pattern = parse_test_pattern_into("24", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(!matches);
    }

    #[test]
    fn test_no_match_different_variables() {
        let (mut pool, expr) = parse_test_expr("x");
        let pattern = parse_test_pattern_into("y", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(!matches);
    }
}

#[cfg(test)]
mod binary_operation_patterns {
    use super::*;

    #[test]
    fn test_simple_addition_pattern() {
        let (mut pool, expr) = parse_test_expr("(x + y)");
        let pattern = parse_test_pattern_into("?a + ?b", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 2);

        let captured_exprs: Vec<_> = captures.values().collect();
        let displays: Vec<_> = captured_exprs
            .iter()
            .map(|c| match c {
                rules::CapturedValue::Expression(e) => pool.display_with_children(*e),
                _ => panic!("Expected expression capture"),
            })
            .collect();

        assert!(displays.contains(&"x".to_string()));
        assert!(displays.contains(&"y".to_string()));
    }

    #[test]
    fn test_repeated_variable_pattern() {
        let (mut pool, expr) = parse_test_expr("(x + x)");
        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 1);

        if let Some(rules::CapturedValue::Expression(captured_expr)) = captures.values().next() {
            assert_expr_display(&pool, *captured_expr, "x");
        } else {
            panic!("Expected captured expression");
        }
    }

    #[test]
    fn test_repeated_variable_no_match() {
        let (mut pool, expr) = parse_test_expr("(x + y)");
        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(!matches);
    }

    #[test]
    fn test_mixed_literal_and_pattern() {
        let (mut pool, expr) = parse_test_expr("(x + 0)");
        let pattern = parse_test_pattern_into("?x + 0", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 1);

        if let Some(rules::CapturedValue::Expression(captured_expr)) = captures.values().next() {
            assert_expr_display(&pool, *captured_expr, "x");
        } else {
            panic!("Expected captured expression");
        }
    }

    #[test]
    fn test_wrong_operation_no_match() {
        let (mut pool, expr) = parse_test_expr("(x * y)");
        let pattern = parse_test_pattern_into("?x + ?y", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(!matches);
    }
}

#[cfg(test)]
mod nested_pattern_matching {
    use super::*;

    #[test]
    fn test_nested_expression_pattern() {
        let (mut pool, expr) = parse_test_expr("((x + y) + z)");
        let pattern = parse_test_pattern_into("(?a + ?b) + ?c", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 3);

        let captured_exprs: Vec<_> = captures.values().collect();
        let displays: Vec<_> = captured_exprs
            .iter()
            .map(|c| match c {
                rules::CapturedValue::Expression(e) => pool.display_with_children(*e),
                _ => panic!("Expected expression capture"),
            })
            .collect();

        assert!(displays.contains(&"x".to_string()));
        assert!(displays.contains(&"y".to_string()));
        assert!(displays.contains(&"z".to_string()));
    }

    #[test]
    fn test_pattern_matches_subexpression() {
        let (mut pool, expr) = parse_test_expr("((x + x) + y)");
        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);

        let children = get_children_vec(&pool, expr);
        let subexpr = children[1];

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, subexpr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 1);
    }
}

#[cfg(test)]
mod function_pattern_matching {
    use super::*;

    #[test]
    fn test_function_call_pattern() {
        let (mut pool, expr) = parse_test_expr("sin(x)");
        let pattern = parse_test_pattern_into("sin(?x)", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 1);

        if let Some(rules::CapturedValue::Expression(captured_expr)) = captures.values().next() {
            assert_expr_display(&pool, *captured_expr, "x");
        } else {
            panic!("Expected captured expression");
        }
    }

    #[test]
    fn test_function_call_wrong_name() {
        let (mut pool, expr) = parse_test_expr("sin(x)");
        let pattern = parse_test_pattern_into("cos(?x)", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(!matches);
    }

    #[test]
    fn test_variable_function_pattern() {
        let (mut pool, expr) = parse_test_expr("sin(x)");
        let pattern = parse_test_pattern_into("?f(?x)", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 2);

        let has_function_capture = captures
            .values()
            .any(|c| matches!(c, rules::CapturedValue::Function(_)));
        let has_expression_capture = captures
            .values()
            .any(|c| matches!(c, rules::CapturedValue::Expression(_)));

        assert!(has_function_capture);
        assert!(has_expression_capture);
    }
}

#[cfg(test)]
mod complex_pattern_scenarios {
    use super::*;

    #[test]
    fn test_distributive_law_pattern() {
        let (mut pool, expr) = parse_test_expr("(a * (b + c))");
        let pattern = parse_test_pattern_into("?x * (?y + ?z)", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 3);

        let captured_exprs: Vec<_> = captures.values().collect();
        let displays: Vec<_> = captured_exprs
            .iter()
            .map(|c| match c {
                rules::CapturedValue::Expression(e) => pool.display_with_children(*e),
                _ => panic!("Expected expression capture"),
            })
            .collect();

        assert!(displays.contains(&"a".to_string()));
        assert!(displays.contains(&"b".to_string()));
        assert!(displays.contains(&"c".to_string()));
    }

    #[test]
    fn test_commutative_matching() {
        let (mut pool1, expr1) = parse_test_expr("(a + b)");
        let pattern = parse_test_pattern_into("?x + ?y", &mut pool1);

        let mut captures1 = HashMap::new();
        let matches1 = pool1.pattern_matches(pattern, expr1, &mut captures1);

        assert!(matches1);

        let (mut pool2, expr2) = parse_test_expr("(b + a)");
        let pattern2 = parse_test_pattern_into("?x + ?y", &mut pool2);
        let mut captures2 = HashMap::new();
        let matches2 = pool2.pattern_matches(pattern2, expr2, &mut captures2);

        assert!(matches2);

        assert_eq!(captures1.len(), 2);
        assert_eq!(captures2.len(), 2);
    }

    #[test]
    fn test_complex_repeated_pattern() {
        let (mut pool, expr) = parse_test_expr("((x + y) + (x + y))");
        let pattern = parse_test_pattern_into("?expr + ?expr", &mut pool);

        let mut captures = HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 1);

        if let Some(rules::CapturedValue::Expression(captured_expr)) = captures.values().next() {
            assert_expr_display(&pool, *captured_expr, "(x + y)");
        } else {
            panic!("Expected captured expression");
        }
    }
}
