#[cfg(test)]
mod tests {
    use crate::*;
    use crate::parser::expr::parse_expression;
    use crate::parser::rules::parse_ruleset;
    use crate::rules::CapturedValue;

    #[test]
    fn test_debug_last_field() {
        let mut pool = Pool::new();
        
        // Simple test: (a+b)*c
        let expr = parse_expression("(a+b)*c", &mut pool).unwrap();
        
        println!("\n=== Initial pool state ===");
        dump_pool(&pool);
        
        // Parse a simple rule that doesn't change structure much
        let ruleset = r#"test {
  swap: ?x * ?y => y * x
}"#;
        parse_ruleset(ruleset, &mut pool).unwrap();
        
        // Apply the rule
        let matches = pool.find_matches(expr);
        if let Some(m) = matches.first() {
            println!("\n=== Before apply_rule ===");
            println!("Applying rule to: {}", pool.display_with_children(m.offset));
            
            // Let's trace through apply_rule step by step
            let rule = pool[m.rule_id];
            
            // Step 1: Build replacement
            let mut replacement_vec = Vec::new();
            pool.build_action_simple(rule.action, &m.captures, &mut replacement_vec);
            
            println!("\n=== Replacement vector ===");
            for (i, (node, _)) in replacement_vec.iter().enumerate() {
                println!("  [{}]: {:?}", i, node);
            }
            
            // Step 2: Copy root expression
            let mut root_vec = Vec::new();
            pool.copy_expression_to_vec(m.root, &mut root_vec);
            
            println!("\n=== Root vector after copy ===");
            for (i, (node, _)) in root_vec.iter().enumerate() {
                println!("  [{}]: {:?}", i, node);
            }
            
            // The actual apply would splice and fix indices here
            let result = pool.apply_rule(m).unwrap();
            
            println!("\n=== After apply_rule ===");
            dump_pool(&pool);
            println!("\nResult: {}", pool.display_with_children(result));
        }
    }
    
    #[test]
    fn test_distribution_last_field() {
        let mut pool = Pool::new();
        
        // Test right distribution: (x+y)*z => (x*z)+(y*z)
        let expr = parse_expression("(x+y)*z", &mut pool).unwrap();
        
        println!("\n=== Initial expression: {} ===", pool.display_with_children(expr));
        dump_pool(&pool);
        
        let ruleset = r#"test {
  right_distrib: (?x + ?y) * ?z => (x * z) + (y * z)
}"#;
        parse_ruleset(ruleset, &mut pool).unwrap();
        
        let matches = pool.find_matches(expr);
        if let Some(m) = matches.first() {
            // Build replacement manually to debug
            let rule = pool[m.rule_id];
            let mut replacement_vec = Vec::new();
            
            println!("\n=== Building action step by step ===");
            println!("Action structure: {}", pool.display_with_children(rule.action));
            
            pool.build_action_simple(rule.action, &m.captures, &mut replacement_vec);
            
            println!("\n=== Replacement vector ===");
            for (i, (node, _)) in replacement_vec.iter().enumerate() {
                println!("  [{}]: {:?}", i, node);
                match node {
                    ExprNode::Call { last, arity, .. } | ExprNode::Struct { last, arity, .. } => {
                        if *arity > 0 {
                            println!("       -> last={}, should point to child at position {}", last, i - last);
                        }
                    }
                    _ => {}
                }
            }
            
            let result = pool.apply_rule(m).unwrap();
            
            println!("\n=== Final pool state ===");
            dump_pool(&pool);
            
            println!("\n=== Result structure ===");
            debug_expr_structure(&pool, result, 0);
        }
    }
    
    fn dump_pool(pool: &Pool) {
        println!("Pool has {} expressions", pool.exprs.len());
        for i in 0..pool.exprs.len() {
            println!("  [{}]: {:?}", i, pool.exprs[i]);
            match &pool.exprs[i] {
                ExprNode::Call { last, arity, .. } | ExprNode::Struct { last, arity, .. } => {
                    if *arity > 0 {
                        let first_child_pos = i.saturating_sub(*last);
                        println!("       -> last={}, first child at position {}", last, first_child_pos);
                    }
                }
                _ => {}
            }
        }
    }
    
    fn debug_expr_structure(pool: &Pool, expr: ExprId, indent: usize) {
        let prefix = "  ".repeat(indent);
        let node = pool[expr];
        
        println!("{}[{}]: {:?}", prefix, expr.0, node);
        
        match node {
            ExprNode::Call { fun, arity, last } => {
                println!("{}  Function: {}, Arity: {}, Last: {}", 
                         prefix, pool.display_function(fun), arity, last);
                let expected_first_child = expr.0.saturating_sub(last);
                println!("{}  First child should be at position: {}", prefix, expected_first_child);
                
                let children: Vec<_> = pool.children(expr).collect();
                println!("{}  Children from iterator: {:?}", prefix, children);
                
                for (i, child) in children.iter().enumerate() {
                    println!("{}  Child[{}]:", prefix, i);
                    debug_expr_structure(pool, *child, indent + 1);
                }
            }
            ExprNode::Variable(v) => {
                println!("{}  Variable: {}", prefix, pool.display_name(v));
            }
            _ => {}
        }
    }
}