mod common;

use common::*;
use expression_explorer::children::Children;
use expression_explorer::*;

#[cfg(test)]
mod parser_bug_fixes {
    use super::*;

    #[test]
    fn test_parser_last_field_calculation() {
        let (pool, expr) = parse_test_expr("(x + y)");

        assert_expr_display(&pool, expr, "(x + y)");

        let children = get_children_vec(&pool, expr);
        assert_eq!(children.len(), 2);

        let left = children[1];
        let right = children[0];

        assert_expr_display(&pool, left, "x");
        assert_expr_display(&pool, right, "y");
    }

    #[test]
    fn test_complex_nested_expression_parsing() {
        let (pool, expr) = parse_test_expr("((x+x)+(x+x)) + ((x+x)+(x+x))");

        assert!(
            pool.exprs.len() < 50,
            "Too many nodes: {}",
            pool.exprs.len()
        );

        let display = pool.display_with_children(expr);
        assert_eq!(display, "(((x + x) + (x + x)) + ((x + x) + (x + x)))");

        assert_eq!(count_nodes(&pool, expr), pool.length(expr));
    }

    #[test]
    fn test_children_iterator_underflow_fix() {
        let (pool, expr) = parse_test_expr("(a + b)");

        let children: Vec<_> = pool.children(expr).collect();
        assert_eq!(children.len(), 2);

        for child in children {
            let _display = pool.display_with_children(child);
        }
    }

    #[test]
    fn test_display_reversal_bug_fix() {
        let (pool, expr) = parse_test_expr("((a + b) + c)");

        assert_expr_display(&pool, expr, "((a + b) + c)");

        let children = get_children_vec(&pool, expr);
        let left_child = children[1];
        assert_expr_display(&pool, left_child, "(a + b)");

        let right_child = children[0];
        assert_expr_display(&pool, right_child, "c");
    }

    #[test]
    fn test_parser_no_garbage_creation() {
        let (mut pool, _expr) = parse_test_expr("(x + x)");

        let x_count = pool.names.iter().filter(|name| name == &"x").count();
        assert_eq!(x_count, 1, "Parser created duplicate variable names");

        assert_eq!(
            pool.exprs.len(),
            3,
            "Parser created wrong number of expression nodes"
        );
    }

    #[test]
    fn test_operator_precedence_consistency() {
        let (pool1, expr1) = parse_test_expr("x + y * z");
        assert_expr_display(&pool1, expr1, "(x + (y * z))");

        let (pool2, expr2) = parse_test_expr("(x + y) * z");
        assert_expr_display(&pool2, expr2, "((x + y) * z)");

        assert_ne!(
            pool1.display_with_children(expr1),
            pool2.display_with_children(expr2)
        );
    }
}

#[cfg(test)]
mod pattern_matching_bug_fixes {
    use super::*;

    #[test]
    fn test_pattern_expression_same_pool_requirement() {
        let (mut pool, expr) = parse_test_expr("(x + y)");
        let pattern = parse_test_pattern_into("?a + ?b", &mut pool);

        let mut captures = std::collections::HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 2);
    }

    #[test]
    fn test_pattern_matching_with_repeated_variables() {
        let (mut pool, expr) = parse_test_expr("(x + x)");
        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);

        let mut captures = std::collections::HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 1);
    }

    #[test]
    fn test_nested_pattern_matching() {
        let (mut pool, expr) = parse_test_expr("((x + y) + z)");
        let pattern = parse_test_pattern_into("(?a + ?b) + ?c", &mut pool);

        let mut captures = std::collections::HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);

        assert!(matches);
        assert_eq!(captures.len(), 3);
    }
}

#[cfg(test)]
mod rule_application_bug_fixes {
    use super::*;

    #[test]
    fn test_rule_application_with_proper_substitution() {
        let (mut pool, expr) = parse_test_expr("(x + x)");
        let pattern = parse_test_pattern_into("?x + ?x", &mut pool);
        let action = parse_test_action_into("2 * x", &mut pool);

        let rule_name = pool.intern_string("double_add".to_string());
        let rule = Rule {
            name: rule_name,
            pattern,
            action,
        };
        let rule_id = pool.add_rule(rule);

        let mut captures = std::collections::HashMap::new();
        let matches = pool.pattern_matches(pattern, expr, &mut captures);
        assert!(matches);

        let match_obj = expression_explorer::rules::Match {
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
    fn test_multiple_rules_no_interference() {
        let (mut pool, expr) = parse_test_expr("(x + 0)");

        let pattern1 = parse_test_pattern_into("?x + 0", &mut pool);
        let action1 = parse_test_action_into("x", &mut pool);
        let rule1_name = pool.intern_string("identity_add".to_string());
        let rule1 = Rule {
            name: rule1_name,
            pattern: pattern1,
            action: action1,
        };
        let rule1_id = pool.add_rule(rule1);

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
        let matching_rule_ids: std::collections::HashSet<_> =
            matches.iter().map(|m| m.rule_id).collect();

        assert!(matching_rule_ids.contains(&rule1_id));
        assert_eq!(matching_rule_ids.len(), 1);
    }
}

#[cfg(test)]
mod memory_and_performance_fixes {
    use super::*;

    #[test]
    fn test_no_excessive_string_duplication() {
        let mut pool = new_test_pool();

        for _ in 0..10 {
            make_var(&mut pool, "test_var");
        }

        let test_var_count = pool.names.iter().filter(|name| name == &"test_var").count();
        assert_eq!(test_var_count, 1);
    }

    #[test]
    fn test_no_excessive_function_duplication() {
        let mut pool = new_test_pool();
        let initial_functions = pool.functions.len();

        for _ in 0..10 {
            pool.intern_function(Function::Add);
        }

        assert_eq!(pool.functions.len(), initial_functions);
    }

    #[test]
    fn test_expression_copying_preserves_structure() {
        let (mut pool, expr) = parse_test_expr("(a + b)");

        let original_display = pool.display_with_children(expr);

        let expr_slice: Vec<ExprNode> = pool.get_full_slice(expr).to_vec();

        for node in expr_slice {
            pool.exprs.push(node);
        }

        let copied_expr = ExprId(pool.exprs.len() - 1);
        pool.mark_expr_end(copied_expr);

        let copied_display = pool.display_with_children(copied_expr);
        assert_eq!(original_display, copied_display);
    }

    #[test]
    fn test_large_expression_reasonable_size() {
        let complex_expr = "((((a+b)+(c+d))+((e+f)+(g+h)))+(((i+j)+(k+l))+((m+n)+(o+p))))";
        let (pool, _expr) = parse_test_expr(complex_expr);

        assert!(
            pool.exprs.len() < 100,
            "Expression too large: {} nodes",
            pool.exprs.len()
        );

        let unique_names: std::collections::HashSet<_> = pool.names.iter().collect();
        assert!(unique_names.len() <= 16);
    }
}

#[cfg(test)]
mod tree_navigation_fixes {
    use super::*;

    #[test]
    fn test_parent_finding_works_correctly() {
        let (pool, expr) = parse_test_expr("(x + y)");

        let children = get_children_vec(&pool, expr);
        let left_child = children[1];
        let right_child = children[0];

        assert_eq!(pool.parent(left_child), Some(expr));
        assert_eq!(pool.parent(right_child), Some(expr));

        assert_eq!(pool.parent(expr), None);
    }

    #[test]
    fn test_root_finding_efficient() {
        let (pool, expr) = parse_test_expr("((a + b) + c)");

        let children = get_children_vec(&pool, expr);
        let nested_expr = children[1];
        let nested_children = get_children_vec(&pool, nested_expr);
        let deep_child = nested_children[1];

        assert_eq!(pool.find_root(expr), Some(expr));
        assert_eq!(pool.find_root(nested_expr), Some(expr));
        assert_eq!(pool.find_root(deep_child), Some(expr));
    }

    #[test]
    fn test_siblings_iterator_works() {
        let (pool, expr) = parse_test_expr("(x + y)");

        let children = get_children_vec(&pool, expr);
        let left = children[1];
        let right = children[0];

        let left_siblings: Vec<_> = pool.siblings(left).collect();
        assert!(left_siblings.contains(&right));

        let right_siblings: Vec<_> = pool.siblings(right).collect();
        assert!(right_siblings.contains(&left));
    }

    #[test]
    fn test_length_calculation_correct() {
        let (pool, simple) = parse_test_expr("x");
        assert_eq!(pool.length(simple), 1);

        let (pool, binary) = parse_test_expr("(x + y)");
        assert_eq!(pool.length(binary), 3);

        let (pool, nested) = parse_test_expr("((a + b) + c)");
        assert_eq!(pool.length(nested), 5);

        let (pool, complex) = parse_test_expr("(((x + y) + z) + w)");
        assert_eq!(pool.length(complex), 7);
    }
}
