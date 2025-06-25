use crate::ast::Rule;
use crate::parser::actions::parse_action;
use crate::parser::patterns::parse_pattern;
use crate::{Location, Pool, RuleId, Ruleset, RulesetId};
use pest::Parser;
use pest::iterators::Pair;

use crate::parser::ruleset_parser::{Rule as PestRule, RulesetParser};

pub fn parse_ruleset(input: &str, pool: &mut Pool) -> Result<RulesetId, String> {
    let pairs = RulesetParser::parse(PestRule::ruleset, input)
        .map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs.into_iter().next().unwrap();

    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str();
    let name_id = pool.intern_string(name.to_string());

    let rules_start = pool.get_rules_len();

    for rule_pair in inner {
        parse_rule_pair(rule_pair, pool)?;
    }

    let rules_end = pool.get_rules_len();

    let ruleset = Ruleset {
        name: name_id,
        rules_start,
        rules_end,
    };

    Ok(pool.add_ruleset(ruleset))
}

pub fn parse_rule_pair(pair: Pair<PestRule>, pool: &mut Pool) -> Result<RuleId, String> {
    let span = pair.as_span();
    let location = Location::new(span.start(), span.end());

    let mut inner = pair.into_inner();
    let rule_name = inner.next().unwrap().as_str();
    let pattern_str = inner.next().unwrap().as_str();
    let action_str = inner.next().unwrap().as_str();

    let pattern_id = parse_pattern(pattern_str, pool)?;

    let action_id = parse_action(action_str, pool)?;

    let name_id = pool.intern_string(rule_name.to_string());

    let rule = Rule {
        name: name_id,
        pattern: pattern_id,
        action: action_id,
    };

    Ok(pool.add_rule_with_location(rule, location))
}
