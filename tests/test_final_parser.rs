#[cfg(test)]
#[test]
fn test_final_parser_verification() {
    use expression_explorer::children::Children;
    use expression_explorer::display::DisplayNode;
    use expression_explorer::parser::*;
    use expression_explorer::*;

    let mut pool = Pool::new();

    println!("=== Final Parser Verification ===\n");

    let expr = "(a + b) + (c + d)";
    let id = parse_expression(expr, &mut pool).unwrap();

    println!("Expression: {}", expr);
    println!("Parsed as: {}", pool.display_with_children(id));
    println!("Expected: ((a + b) + (c + d))");

    println!("Root children: {:?}", pool.children(id).collect::<Vec<_>>());

    let children: Vec<_> = pool.children(id).collect();
    assert_eq!(children.len(), 2);

    let left_child = children[1];
    let right_child = children[0];

    println!(
        "Left child: {} -> {}",
        left_child.0,
        pool.display_with_children(left_child)
    );
    println!(
        "Right child: {} -> {}",
        right_child.0,
        pool.display_with_children(right_child)
    );

    println!("\n--- Testing pattern matching ---");
    let simple_expr = "(x + x)";
    let simple_id = parse_expression(simple_expr, &mut pool).unwrap();
    println!(
        "Simple expression: {} -> {}",
        simple_expr,
        pool.display_with_children(simple_id)
    );

    let ruleset_text = "test_rules {\n  double: ?x + ?x => 2 * x\n}";
    let _ruleset_id = parse_ruleset(ruleset_text, &mut pool).unwrap();

    let matches = pool.find_matches(simple_id);
    println!("Found {} matches", matches.len());

    for (i, match_) in matches.iter().enumerate() {
        println!(
            "Match {}: at {:?} -> {}",
            i + 1,
            match_.root,
            pool.display_with_children(match_.root)
        );
        for (name_id, captured_value) in &match_.captures {
            if let expression_explorer::rules::CapturedValue::Expression(expr_id) = captured_value {
                println!(
                    "  NameId({:?}) captures ExprId({}) = {}",
                    name_id.0,
                    expr_id.0,
                    pool.display_with_children(*expr_id)
                );
            }
        }

        if let Some(result) = pool.apply_rule(match_) {
            println!("  Result: {}", pool.display_with_children(result));
        }
    }
}
