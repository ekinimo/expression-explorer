pub mod expr_parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "parser/expressions.pest"]
    pub struct ExprParser;
}

pub mod pattern_parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "parser/patterns.pest"]
    pub struct PatternParser;
}

pub mod action_parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "parser/actions.pest"]
    pub struct ActionParser;
}

pub mod compute_parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "parser/compute.pest"]
    pub struct ComputeParser;
}

pub mod ruleset_parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "parser/rulesets.pest"]
    pub struct RulesetParser;
}

pub mod actions;
pub mod compute;
pub mod expr;
pub mod patterns;
pub mod rules;

pub use actions::parse_action_pair;
pub use compute::parse_compute_expr;
pub use expr::*;
pub use patterns::parse_pattern_pair;
pub use rules::*;
