#[cfg(test)]
mod tests {
    use crate::*;
    use crate::parser::expr::parse_expression;
    use crate::parser::rules::parse_ruleset;
    use crate::rules::CapturedValue;

    #[test]
    fn test_debug_simple_case() {
        let mut pool = Pool::new();
        
        // Parse a simple expression (a+b)*c
        let expr = parse_expression("(a+b)*c", &mut pool).unwrap();
        
        println!("\n=== Pool state after parsing (a+b)*c ===");
        for i in 0..pool.exprs.len() {
            let expr_id = ExprId(i);
            println!("  [{}]: {:?}", i, pool.exprs[i]);
        }
        
        // Check the structure
        let root = pool[expr];
        match root {
            ExprNode::Call { fun, arity, last } => {
                println!("\nRoot node (index {}): function={}, arity={}, last={}", 
                         expr.0, pool.display_function(fun), arity, last);
                
                // The children iterator logic
                let node_idx = expr.0;  // 4
                let first_child_pos = node_idx - last;  // 4 - 4 = 0
                println!("First child position: {} - {} = {}", node_idx, last, first_child_pos);
                
                // So children are at positions 0..4
                // But we want positions 2 and 3 (the two direct children)
                
                let children: Vec<_> = pool.children(expr).collect();
                println!("Children from iterator: {:?}", children);
                
                // Let's manually check what we expect
                // For (a+b)*c stored as [a, b, (a+b), c, (a+b)*c]
                // The * node at position 4 should have children at positions 2 and 3
                println!("\nExpected children:");
                println!("  Position 2: {:?} = {}", pool.exprs[2], pool.display_with_children(ExprId(2)));
                println!("  Position 3: {:?} = {}", pool.exprs[3], pool.display_with_children(ExprId(3)));
            }
            _ => panic!("Expected Call node"),
        }
    }

    #[test] 
    fn test_rule_application_debug() {
        let mut pool = Pool::new();
        
        // Parse (x+y)*z
        let expr = parse_expression("(x+y)*z", &mut pool).unwrap();
        println!("\n=== Original expression: {} ===", pool.display_with_children(expr));
        dump_pool(&pool, "After parsing");
        
        // Parse right_distrib rule
        let ruleset = r#"algebra {
  right_distrib: (?x + ?y) * ?z => (x * z) + (y * z)
}"#;
        parse_ruleset(ruleset, &mut pool).unwrap();
        
        // Find and apply the rule
        let matches = pool.find_matches(expr);
        let m = matches.iter().find(|m| {
            let rule = pool[m.rule_id];
            pool.display_name(rule.name) == "right_distrib"
        }).unwrap();
        
        println!("\nMatch found at offset: {:?}", m.offset);
        println!("Captures:");
        for (name_id, value) in &m.captures {
            match value {
                CapturedValue::Expression(e) => {
                    println!("  ?{} = {} (ExprId({}))", pool[*name_id], pool.display_with_children(*e), e.0);
                }
                _ => {}
            }
        }
        
        // Now let's trace through apply_rule
        println!("\n=== Applying rule ===");
        
        // The rule action is: (x * z) + (y * z)
        // Let's see what build_action_simple creates
        let rule = pool[m.rule_id];
        println!("Rule action: {}", pool.display_with_children(rule.action));
        
        let result = pool.apply_rule(m).unwrap();
        dump_pool(&pool, "After applying rule");
        
        println!("\nResult: {} (ExprId({}))", pool.display_with_children(result), result.0);
        
        // Check the structure of the result
        println!("\n=== Result structure ===");
        debug_expr_structure(&pool, result, 0);
    }

    #[test]
    fn test_nested_application_debug() {
        let mut pool = Pool::new();
        
        // Start with ((x+y)*x)+((x+y)*y) to see what happens with nested application
        let expr = parse_expression("((x+y)*x)+((x+y)*y)", &mut pool).unwrap();
        println!("\n=== Original: {} ===", pool.display_with_children(expr));
        dump_pool(&pool, "Initial");
        
        // Add right_distrib rule
        let ruleset = r#"algebra {
  right_distrib: (?x + ?y) * ?z => (x * z) + (y * z)
}"#;
        parse_ruleset(ruleset, &mut pool).unwrap();
        
        // Find all matches
        let matches = pool.find_matches(expr);
        println!("\nFound {} matches", matches.len());
        for (i, m) in matches.iter().enumerate() {
            println!("  Match {}: offset={} ({})", i, m.offset.0, pool.display_with_children(m.offset));
        }
        
        // Apply to second match (x+y)*y
        let second_match = &matches[1];
        println!("\nApplying rule to: {}", pool.display_with_children(second_match.offset));
        println!("Root: {}, Offset: {}", second_match.root.0, second_match.offset.0);
        
        let result = pool.apply_rule(second_match).unwrap();
        dump_pool(&pool, "After applying to (x+y)*y");
        
        println!("\nResult: {}", pool.display_with_children(result));
        debug_expr_structure(&pool, result, 0);
    }
    
    fn dump_pool(pool: &Pool, label: &str) {
        println!("\n=== {} ===", label);
        println!("Pool has {} expressions", pool.exprs.len());
        for i in 0..pool.exprs.len() {
            let expr_id = ExprId(i);
            println!("  [{}]: {:?}", i, pool.exprs[i]);
            if let ExprNode::Call { last, .. } | ExprNode::Struct { last, .. } = pool.exprs[i] {
                println!("       -> last={}, children would start at {}", last, i.saturating_sub(last));
            }
        }
    }
    
    fn debug_expr_structure(pool: &Pool, expr: ExprId, indent: usize) {
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
                    debug_expr_structure(pool, *child, indent + 1);
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