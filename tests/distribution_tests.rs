use expression_explorer::*;
use expression_explorer::parser::expr::parse_expression;
use expression_explorer::parser::rules::parse_ruleset;

#[test]
fn test_multiple_distribution_steps() {
    let mut pool = Pool::new();
    
    // Parse the expression (x+y)*(x+y)
    let expr = parse_expression("(x+y)*(x+y)", &mut pool).unwrap();
    let expr_str = pool.display_with_children(expr);
    assert_eq!(expr_str, "((x + y) * (x + y))");
    
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
    assert_eq!(step2_str, "(((x * x) + (y * x)) + ((x + y) * y))");
    
    // Step 3: Apply right_distrib to (x+y)*y
    let matches3 = pool.find_matches(step2);
    
    let right_match2 = matches3.iter().find(|m| {
        let rule = pool[m.rule_id];
        pool.display_name(rule.name) == "right_distrib" &&
        pool.display_with_children(m.offset).contains("(x + y) * y")
    }).expect("Should find right_distrib for (x+y)*y");
    
    let final_result = pool.apply_rule(right_match2).unwrap();
    let final_str = pool.display_with_children(final_result);
    
    // Final result should be ((x*x) + (y*x)) + ((x*y) + (y*y))
    assert_eq!(final_str, "(((x * x) + (y * x)) + ((x * y) + (y * y)))");
}

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