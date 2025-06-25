mod common;

use common::*;
use expression_explorer::children::Children;
use expression_explorer::*;

#[cfg(test)]
mod basic_pool_operations {
    use super::*;

    #[test]
    fn test_new_pool() {
        let pool = new_test_pool();

        assert!(pool.functions.len() > 0);

        assert_eq!(pool.exprs.len(), 0);
        assert_eq!(pool.patterns.len(), 0);
        assert_eq!(pool.actions.len(), 0);
        assert_eq!(pool.rules.len(), 0);
    }

    #[test]
    fn test_string_interning() {
        let mut pool = new_test_pool();

        let id1 = pool.intern_string("test".to_string());
        let id2 = pool.intern_string("test".to_string());
        let id3 = pool.intern_string("different".to_string());

        assert_eq!(id1, id2);

        assert_ne!(id1, id3);

        assert_eq!(&pool[id1], "test");
        assert_eq!(&pool[id3], "different");
    }

    #[test]
    fn test_function_interning() {
        let mut pool = new_test_pool();

        let add1 = pool.intern_function(Function::Add);
        let add2 = pool.intern_function(Function::Add);
        let mult = pool.intern_function(Function::Multiply);

        assert_eq!(add1, add2);

        assert_ne!(add1, mult);
    }

    #[test]
    fn test_expression_creation() {
        let mut pool = new_test_pool();

        let x = make_var(&mut pool, "x");
        let y = make_var(&mut pool, "y");

        assert_eq!(count_nodes(&pool, x), 1);
        assert_eq!(count_nodes(&pool, y), 1);

        assert_expr_display(&pool, x, "x");
        assert_expr_display(&pool, y, "y");
    }

    #[test]
    fn test_expression_ends_tracking() {
        let mut pool = new_test_pool();

        let x = make_var(&mut pool, "x");
        let y = make_var(&mut pool, "y");

        assert!(pool.is_root(x));
        assert!(pool.is_root(y));

        let roots: Vec<_> = pool.get_all_roots().collect();
        assert!(roots.contains(&x));
        assert!(roots.contains(&y));
    }
}

#[cfg(test)]
mod tree_navigation_tests {
    use super::*;

    #[test]
    fn test_simple_parent_child() {
        let (pool, expr) = parse_test_expr("(x + y)");

        let children = get_children_vec(&pool, expr);
        assert_eq!(children.len(), 2);

        let left = children[1];
        let right = children[0];

        assert_eq!(pool.parent(left), Some(expr));
        assert_eq!(pool.parent(right), Some(expr));

        assert_eq!(pool.parent(expr), None);
    }

    #[test]
    fn test_find_root() {
        let (pool, expr) = parse_test_expr("(x + y)");

        let children = get_children_vec(&pool, expr);
        let left = children[1];
        let right = children[0];

        assert_eq!(pool.find_root(expr), Some(expr));
        assert_eq!(pool.find_root(left), Some(expr));
        assert_eq!(pool.find_root(right), Some(expr));
    }

    #[test]
    fn test_nested_tree_navigation() {
        let (pool, expr) = parse_test_expr("((a + b) + c)");

        let children = get_children_vec(&pool, expr);
        let inner_expr = children[1];

        assert_expr_display(&pool, inner_expr, "(a + b)");

        assert_eq!(pool.parent(inner_expr), Some(expr));

        let inner_children = get_children_vec(&pool, inner_expr);
        let a_node = inner_children[1];

        assert_expr_display(&pool, a_node, "a");
        assert_eq!(pool.find_root(a_node), Some(expr));

        let ancestors: Vec<_> = pool.ancestors(a_node).collect();
        assert!(ancestors.contains(&inner_expr));
        assert!(ancestors.contains(&expr));
    }

    #[test]
    fn test_siblings() {
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
    fn test_length_calculation() {
        let (pool, simple) = parse_test_expr("x");
        assert_eq!(pool.length(simple), 1);

        let (pool, binary) = parse_test_expr("(x + y)");
        assert_eq!(pool.length(binary), 3);

        let (pool, nested) = parse_test_expr("((a + b) + c)");
        assert_eq!(pool.length(nested), 5);
    }
}

#[cfg(test)]
mod expression_copying_tests {
    use super::*;

    #[test]
    fn test_get_full_slice() {
        let (pool, expr) = parse_test_expr("(x + y)");

        let slice = pool.get_full_slice(expr);
        assert_eq!(slice.len(), 3);

        if let [first, second, third] = slice {
            assert!(matches!(first, ExprNode::Variable(_)));
            assert!(matches!(second, ExprNode::Variable(_)));
            assert!(matches!(third, ExprNode::Call { .. }));
        } else {
            panic!("Expected exactly 3 nodes");
        }
    }

    #[test]
    fn test_expression_copying_preserves_structure() {
        let (mut pool, expr1) = parse_test_expr("(x + y)");

        let original_display = pool.display_with_children(expr1);

        let expr_slice: Vec<ExprNode> = pool.get_full_slice(expr1).to_vec();

        for node in expr_slice {
            pool.exprs.push(node);
        }

        let expr2 = ExprId(pool.exprs.len() - 1);
        pool.mark_expr_end(expr2);

        assert_eq!(pool.display_with_children(expr2), original_display);
    }
}

#[cfg(test)]
mod memory_efficiency_tests {
    use super::*;

    #[test]
    fn test_no_unnecessary_string_duplication() {
        let mut pool = new_test_pool();

        for _ in 0..10 {
            make_var(&mut pool, "x");
        }

        let x_count = pool.names.iter().filter(|s| s == &"x").count();
        assert_eq!(x_count, 1);
    }

    #[test]
    fn test_no_unnecessary_function_duplication() {
        let mut pool = new_test_pool();

        let initial_functions = pool.functions.len();

        for _ in 0..10 {
            pool.intern_function(Function::Add);
        }

        assert_eq!(pool.functions.len(), initial_functions);
    }

    #[test]
    fn test_reasonable_expression_size() {
        let (pool, _expr) = parse_test_expr("((x+x)+(x+x)) + ((x+x)+(x+x))");

        assert!(
            pool.exprs.len() < 50,
            "Too many expression nodes: {}",
            pool.exprs.len()
        );

        let x_count = pool.names.iter().filter(|s| s == &"x").count();
        assert_eq!(x_count, 1);
    }
}
