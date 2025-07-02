#[cfg(test)]
mod tests {
    use crate::*;
    use crate::parser::expr::parse_expression;
    use crate::parser::rules::parse_ruleset;

    #[test]
    fn test_left_distribution() {
        let mut pool = Pool::new();
        
        // Parse the expression (x+y)*(x+y)
        let expr = parse_expression("(x+y)*(x+y)", &mut pool).unwrap();
        let expr_str = pool.display_with_children(expr);
        assert_eq!(expr_str, "((x + y) * (x + y))");
        
        // Parse left distribution rule
        let ruleset = r#"algebra {
  left_distrib: ?x * (?y + ?z) => (x * y) + (x * z)
}"#;
        parse_ruleset(ruleset, &mut pool).unwrap();
        
        // Find matches
        let matches = pool.find_matches(expr);
        assert!(!matches.is_empty(), "Should find at least one match");
        
        // Find left_distrib match
        let left_distrib_match = matches.iter().find(|m| {
            let rule = pool[m.rule_id];
            pool.display_name(rule.name) == "left_distrib"
        });
        
        assert!(left_distrib_match.is_some(), "Should find left_distrib match");
        
        let m = left_distrib_match.unwrap();
        
        // Verify captures
        assert_eq!(m.captures.len(), 3, "Should have 3 captures (x, y, z)");
        
        // Apply the rule
        let result = pool.apply_rule(m).unwrap();
        let result_str = pool.display_with_children(result);
        
        // First application should give: ((x+y)*x) + ((x+y)*y)
        assert_eq!(result_str, "(((x + y) * x) + ((x + y) * y))");
    }

    #[test]
    fn test_right_distribution() {
        let mut pool = Pool::new();
        
        // Parse the expression (x+y)*z
        let expr = parse_expression("(x+y)*z", &mut pool).unwrap();
        let expr_str = pool.display_with_children(expr);
        assert_eq!(expr_str, "((x + y) * z)");
        
        // Parse right distribution rule
        let ruleset = r#"algebra {
  right_distrib: (?x + ?y) * ?z => (x * z) + (y * z)
}"#;
        parse_ruleset(ruleset, &mut pool).unwrap();
        
        // Find matches
        let matches = pool.find_matches(expr);
        assert!(!matches.is_empty(), "Should find at least one match");
        
        // Find right_distrib match
        let right_distrib_match = matches.iter().find(|m| {
            let rule = pool[m.rule_id];
            pool.display_name(rule.name) == "right_distrib"
        });
        
        assert!(right_distrib_match.is_some(), "Should find right_distrib match");
        
        let m = right_distrib_match.unwrap();
        
        // Apply the rule
        let result = pool.apply_rule(m).unwrap();
        let result_str = pool.display_with_children(result);
        
        // Should give: (x*z) + (y*z)
        assert_eq!(result_str, "((x * z) + (y * z))");
    }

    #[test]
    fn test_multiple_distribution_steps() {
        let mut pool = Pool::new();
        
        // Parse the expression (x+y)*(x+y)
        let expr = parse_expression("(x+y)*(x+y)", &mut pool).unwrap();
        
        // Parse both distribution rules
        let ruleset = r#"algebra {
  left_distrib: ?x * (?y + ?z) => (x * y) + (x * z)
  right_distrib: (?x + ?y) * ?z => (x * z) + (y * z)
}"#;
        parse_ruleset(ruleset, &mut pool).unwrap();
        
        // Step 1: Apply left_distrib to (x+y)*(x+y)
        let matches1 = pool.find_matches(expr);
        let left_match = matches1.iter().find(|m| {
            let rule = pool[m.rule_id];
            pool.display_name(rule.name) == "left_distrib"
        }).expect("Should find left_distrib");
        
        let step1 = pool.apply_rule(left_match).unwrap();
        let step1_str = pool.display_with_children(step1);
        assert_eq!(step1_str, "(((x + y) * x) + ((x + y) * y))");
        
        // Step 2: Apply right_distrib to (x+y)*x
        let matches2 = pool.find_matches(step1);
        let right_matches: Vec<_> = matches2.iter().filter(|m| {
            let rule = pool[m.rule_id];
            pool.display_name(rule.name) == "right_distrib"
        }).collect();
        
        assert!(right_matches.len() >= 2, "Should find at least 2 right_distrib matches");
        
        // Apply right_distrib to the first (x+y)*x subexpression
        let step2 = pool.apply_rule(right_matches[0]).unwrap();
        let step2_str = pool.display_with_children(step2);
        
        // Now we should have ((x*x) + (y*x)) + ((x+y)*y)
        println!("After first right_distrib: {}", step2_str);
        
        // Step 3: Apply right_distrib to (x+y)*y
        let matches3 = pool.find_matches(step2);
        // Debug: Let's check what's in the pool around the match
        println!("\n=== Pool contents around match ===");
        println!("Match root at ExprId(30), offset at ExprId(29)");
        for i in 20..=30 {
            if let Some(node) = pool.exprs.get(i) {
                println!("  [{}]: {:?} = {}", i, node, pool.display_with_children(ExprId(i)));
            }
        }
        
        println!("\nStep 3: Found {} matches", matches3.len());
        for (i, m) in matches3.iter().enumerate() {
            let rule = pool[m.rule_id];
            println!("  Match {}: {} at offset {} = {}", 
                     i, 
                     pool.display_name(rule.name), 
                     m.offset.0,
                     pool.display_with_children(m.offset));
        }
        
        let right_match2 = matches3.iter().find(|m| {
            let rule = pool[m.rule_id];
            pool.display_name(rule.name) == "right_distrib" &&
            pool.display_with_children(m.offset).contains("(x + y) * y")
        }).expect("Should find right_distrib for (x+y)*y");
        
        println!("\nApplying to match:");
        println!("  Root: ExprId({}) = {}", right_match2.root.0, pool.display_with_children(right_match2.root));
        println!("  Offset: ExprId({}) = {}", right_match2.offset.0, pool.display_with_children(right_match2.offset));
        println!("  Captures:");
        for (name_id, value) in &right_match2.captures {
            if let crate::rules::CapturedValue::Expression(e) = value {
                println!("    ?{} = {} (ExprId({}))", pool[*name_id], pool.display_with_children(*e), e.0);
            }
        }
        
        // Let's trace through the splice calculation manually
        let root_vec_len = pool.get_full_slice(right_match2.root).len();
        let target_slice_len = pool.get_full_slice(right_match2.offset).len();
        let match_start_in_pool = right_match2.offset.0 + 1 - target_slice_len;
        let root_start_in_pool = right_match2.root.0 + 1 - root_vec_len;
        let target_start = match_start_in_pool.saturating_sub(root_start_in_pool);
        let target_end = target_start + target_slice_len;
        
        println!("\nSplice calculation debug:");
        println!("  root_vec_len = {}", root_vec_len);
        println!("  target_slice_len = {}", target_slice_len);
        println!("  match_start_in_pool = {} + 1 - {} = {}", right_match2.offset.0, target_slice_len, match_start_in_pool);
        println!("  root_start_in_pool = {} + 1 - {} = {}", right_match2.root.0, root_vec_len, root_start_in_pool);
        println!("  target_start = {} - {} = {}", match_start_in_pool, root_start_in_pool, target_start);
        println!("  target_end = {} + {} = {}", target_start, target_slice_len, target_end);
        
        // Debug: print what ExprId(25) actually is
        println!("\nChecking captures:");
        println!("  ExprId(25) = {} (should be just 'x')", pool.display_with_children(ExprId(25)));
        println!("  ExprId(26) = {} (should be just 'y')", pool.display_with_children(ExprId(26)));
        println!("  ExprId(28) = {} (should be just 'y')", pool.display_with_children(ExprId(28)));
        
        let final_result = pool.apply_rule(right_match2).unwrap();
        
        println!("\nFinal result: ExprId({})", final_result.0);
        println!("Pool size after: {}", pool.exprs.len());
        
        // Let's check what's at the positions that should be 'x'
        println!("\nChecking pool after transformation:");
        for i in 31..pool.exprs.len() {
            println!("  [{}]: {:?}", i, pool.exprs[i]);
        }
        
        // Let's trace through the structure starting from the root
        println!("\nTracing structure from root:");
        println!("Root at [43] has last=12, so first child at 43-12=31");
        println!("  Child 1 at [42]");
        println!("  Child 2 at [31]");
        
        // Check what's at position 31
        println!("\nPosition 31 is multiplication with last=2, so:");
        println!("  Its children are at 31-2=29 and 30");
        println!("  But wait, positions 29 and 30 are from the ORIGINAL expression!");
        
        // This is the bug! The last field is pointing outside the newly created nodes
        
        let final_str = pool.display_with_children(final_result);
        
        // Final result should be ((x*x) + (y*x)) + ((x*y) + (y*y))
        assert_eq!(final_str, "(((x * x) + (y * x)) + ((x * y) + (y * y)))");
    }

    #[test]
    fn test_expression_structure() {
        let mut pool = Pool::new();
        
        // Create a simple expression to test structure
        let expr = parse_expression("(a+b)*c", &mut pool).unwrap();
        
        // Debug the structure
        println!("\nExpression structure for (a+b)*c:");
        debug_structure(&pool, expr, 0);
        
        // Verify the structure is correct
        let node = pool[expr];
        match node {
            ExprNode::Call { fun, arity, last } => {
                assert_eq!(pool.display_function(fun), "*");
                assert_eq!(arity, 2);
                assert_eq!(last, 4); // Points to the start of the entire subtree
                
                let children: Vec<_> = pool.children(expr).collect();
                assert_eq!(children.len(), 2);
                
                // The iterator returns children in reverse order (right to left)
                // So first child is c, second is (a+b)
                let right_child = children[0];
                let right_str = pool.display_with_children(right_child);
                assert_eq!(right_str, "c");
                
                let left_child = children[1];
                let left_str = pool.display_with_children(left_child);
                assert_eq!(left_str, "(a + b)");
            }
            _ => panic!("Root should be a Call node"),
        }
    }

    #[test]
    fn test_debug_nested_splice() {
        let mut pool = Pool::new();
        
        // Start with ((x*x) + (y*x)) + ((x+y)*y)
        let expr = parse_expression("((x*x) + (y*x)) + ((x+y)*y)", &mut pool).unwrap();
        println!("\n=== Initial expression: {} ===", pool.display_with_children(expr));
        
        // Add right_distrib rule
        let ruleset = r#"test {
  right_distrib: (?x + ?y) * ?z => (x * z) + (y * z)
}"#;
        parse_ruleset(ruleset, &mut pool).unwrap();
        
        // Find the match for (x+y)*y
        let matches = pool.find_matches(expr);
        let m = matches.first().expect("Should find a match");
        
        println!("\nMatch details:");
        println!("  Root: ExprId({}) = {}", m.root.0, pool.display_with_children(m.root));
        println!("  Offset: ExprId({}) = {}", m.offset.0, pool.display_with_children(m.offset));
        
        // Build replacement
        let rule = pool[m.rule_id];
        let mut replacement_vec = Vec::new();
        pool.build_action_simple(rule.action, &m.captures, &mut replacement_vec);
        
        println!("\nReplacement: {} nodes", replacement_vec.len());
        
        // Copy root expression
        let mut root_vec = Vec::new();
        pool.copy_expression_to_vec(m.root, &mut root_vec);
        
        println!("\nRoot expression: {} nodes", root_vec.len());
        for (i, (node, _)) in root_vec.iter().enumerate() {
            println!("  [{}]: {:?}", i, node);
        }
        
        // Calculate splice positions
        let relative_pos = m.root.0 - m.offset.0;
        let target_slice_len = pool.get_full_slice(m.offset).len();
        let root_slice_len = pool.get_full_slice(m.root).len();
        
        let target_end = root_slice_len.saturating_sub(relative_pos);
        let target_start = target_end.saturating_sub(target_slice_len);
        
        println!("\nSplice calculation:");
        println!("  m.root.0 = {}", m.root.0);
        println!("  m.offset.0 = {}", m.offset.0);
        println!("  relative_pos = {} - {} = {}", m.root.0, m.offset.0, relative_pos);
        println!("  target_slice_len (size of match) = {}", target_slice_len);
        println!("  root_slice_len (size of root) = {}", root_slice_len);
        println!("  target_end = {} - {} = {}", root_slice_len, relative_pos, target_end);
        println!("  target_start = {} - {} = {}", target_end, target_slice_len, target_start);
        println!("  => Splice range: {}..{}", target_start, target_end);
        
        // This calculation seems wrong! Let's think about it differently
        // If root is at position 11 and offset is at position 10, relative_pos = 1
        // But that doesn't help us find where in the root_vec the offset expression is
        
        // Let's examine the offset slice in the original pool
        let offset_slice = pool.get_full_slice(m.offset);
        println!("\nOffset slice in pool:");
        for (i, node) in offset_slice.iter().enumerate() {
            println!("  pool[{}]: {:?}", m.offset.0 - offset_slice.len() + 1 + i, node);
        }
    }

    fn debug_structure(pool: &Pool, expr: ExprId, indent: usize) {
        let prefix = "  ".repeat(indent);
        let node = pool[expr];
        
        println!("{}ExprId({}): {:?}", prefix, expr.0, node);
        
        match node {
            ExprNode::Call { fun, arity, last } => {
                println!("{}  Function: {}, Arity: {}, Last: {}", 
                         prefix, pool.display_function(fun), arity, last);
                let children: Vec<_> = pool.children(expr).collect();
                println!("{}  Children ({} total): {:?}", prefix, children.len(), children);
                for (i, child) in children.iter().enumerate() {
                    println!("{}  Child[{}]:", prefix, i);
                    debug_structure(pool, *child, indent + 1);
                }
            }
            ExprNode::Struct { name, arity, last } => {
                println!("{}  Struct: {}, Arity: {}, Last: {}", 
                         prefix, pool.display_name(name), arity, last);
            }
            ExprNode::Variable(v) => {
                println!("{}  Variable: {}", prefix, pool.display_name(v));
            }
            ExprNode::Number(n) => {
                println!("{}  Number: {}", prefix, n);
            }
        }
    }
}