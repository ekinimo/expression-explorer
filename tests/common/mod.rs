use expression_explorer::children::Children;
use expression_explorer::parser::actions::parse_action;
use expression_explorer::parser::patterns::parse_pattern;
use expression_explorer::parser::*;
use expression_explorer::*;

pub fn new_test_pool() -> Pool {
    Pool::new()
}

pub fn parse_test_expr(input: &str) -> (Pool, ExprId) {
    let mut pool = new_test_pool();
    let expr_id = parse_expression(input, &mut pool).expect("Failed to parse test expression");
    (pool, expr_id)
}


pub fn assert_expr_display(pool: &Pool, expr: ExprId, expected: &str) {
    let actual = pool.display_with_children(expr);
    assert_eq!(
        actual, expected,
        "Expression display mismatch:\n  actual: {}\n  expected: {}",
        actual, expected
    );
}

pub fn make_var(pool: &mut Pool, name: &str) -> ExprId {
    let name_id = pool.intern_string(name.to_string());
    let expr_id = pool.add_expr(ExprNode::Variable(name_id));
    pool.mark_expr_end(expr_id);
    expr_id
}

pub fn make_num(pool: &mut Pool, value: i32) -> ExprId {
    let expr_id = pool.add_expr(ExprNode::Number(value));
    pool.mark_expr_end(expr_id);
    expr_id
}

pub fn parse_test_pattern_into(input: &str, pool: &mut Pool) -> PatternId {
    parse_pattern(input, pool).expect("Failed to parse test pattern")
}

pub fn parse_test_action_into(input: &str, pool: &mut Pool) -> ActionId {
    parse_action(input, pool).expect("Failed to parse test action")
}

pub fn get_children_vec(pool: &Pool, expr: ExprId) -> Vec<ExprId> {
    pool.children(expr).collect()
}

pub fn count_nodes(pool: &Pool, expr: ExprId) -> usize {
    let mut count = 1;
    for child in pool.children(expr) {
        count += count_nodes(pool, child);
    }
    count
}

pub fn parse_test_pattern(input: &str) -> (Pool, PatternId) {
    let mut pool = new_test_pool();
    let pattern_id = parse_pattern(input, &mut pool).expect("Failed to parse test pattern");
    (pool, pattern_id)
}

pub fn parse_test_action(input: &str) -> (Pool, ActionId) {
    let mut pool = new_test_pool();
    let action_id = parse_action(input, &mut pool).expect("Failed to parse test action");
    (pool, action_id)
}

pub fn parse_test_ruleset(input: &str) -> (Pool, RulesetId) {
    let mut pool = new_test_pool();
    let ruleset_id = parse_ruleset(input, &mut pool).expect("Failed to parse test ruleset");
    (pool, ruleset_id)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utilities_work() {
        let mut pool = new_test_pool();

        let x = make_var(&mut pool, "x");
        assert_expr_display(&pool, x, "x");

        let two = make_num(&mut pool, 2);
        assert_expr_display(&pool, two, "2");

        let (pool2, expr) = parse_test_expr("(x + y)");
        assert_expr_display(&pool2, expr, "(x + y)");
    }
}
