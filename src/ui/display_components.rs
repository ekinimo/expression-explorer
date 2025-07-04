use super::styles;
use crate::{
    Action, ActionId, Children, DisplayNode, ExprId, ExprNode, Pattern, PatternId, Pool, RuleId,
    RulesetId,
};
use dioxus::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum ViewMode {
    Text,
}

impl ViewMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ViewMode::Text => "text",
        }
    }
}

#[component]
pub fn ExpressionCard(
    pool: Signal<Pool>,
    expr_id: ExprId,
    highlighted_subexpr: Option<ExprId>,
) -> Element {
    let pool_ref = pool.read();

    rsx! {
        div { class: "bg-white rounded-lg shadow-lg border-2 border-blue-200 p-6 mb-6",
            div { class: "flex items-center justify-between mb-4 pb-4 border-b",
                div { class: "flex items-center gap-4",
                    h2 { class: "text-xl font-bold text-gray-800", "Current Expression" }
                    div { class: "flex items-center gap-2",
                        span { class: "px-3 py-1 text-sm bg-blue-100 text-blue-800 rounded-full font-medium",
                            "{get_expr_type_name(&pool_ref[expr_id])}"
                        }
                        span { class: "text-sm text-gray-500", "ID: {expr_id:?}" }
                    }
                }

                // View mode selection removed - only text view available
            }

            div { class: "space-y-6",
                div { class: "space-y-2",
                    div { class: "text-sm font-medium text-gray-700", "Text Representation:" }
                    div { class: "p-4 bg-gray-50 rounded-lg border font-mono text-lg",
                        ExpressionText {
                            pool: pool,
                            expr_id: expr_id,
                            highlighted_subexpr: highlighted_subexpr,
                        }
                    }
                }

                div { class: "flex gap-6 pt-4 border-t text-sm text-gray-600",
                    div { "Children: {pool_ref.children(expr_id).count()}" }
                    div { "Depth: {calculate_expression_depth(&pool_ref, expr_id)}" }
                    div { "Size: {calculate_expression_size(&pool_ref, expr_id)}" }
                }
            }
        }
    }
}

#[component]
pub fn CompactExpressionCard(
    pool: Signal<Pool>,
    expr_id: ExprId,
    highlighted_subexpr: Option<ExprId>,
) -> Element {
    let pool_ref = pool.read();

    rsx! {
        div { class: "bg-white rounded-lg shadow-md border border-gray-200 p-4 max-w-2xl mx-auto",
            div { class: "flex items-center justify-between mb-3",
                div { class: "flex items-center gap-3",
                    h3 { class: "text-lg font-semibold text-gray-800", "Current Expression" }
                    div { class: "flex items-center gap-2",
                        span { class: "px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded font-medium",
                            "{get_expr_type_name(&pool_ref[expr_id])}"
                        }
                        span { class: "text-xs text-gray-500", "({expr_id:?})" }
                    }
                }

                // View mode selection removed - only text view available
            }

            div { class: "space-y-3",
                div { class: "p-3 bg-gray-50 rounded border font-mono text-sm max-h-20 overflow-auto",
                    ExpressionText {
                        pool: pool,
                        expr_id: expr_id,
                        highlighted_subexpr: highlighted_subexpr,
                    }
                }

                div { class: "flex gap-4 pt-2 border-t text-xs text-gray-500",
                    div { "Children: {pool_ref.children(expr_id).count()}" }
                    div { "Depth: {calculate_expression_depth(&pool_ref, expr_id)}" }
                    div { "Size: {calculate_expression_size(&pool_ref, expr_id)}" }
                }
            }
        }
    }
}

#[component]
fn ExpressionText(
    pool: Signal<Pool>,
    expr_id: ExprId,
    highlighted_subexpr: Option<ExprId>,
) -> Element {
    let pool_ref = pool.read();

    if let Some(highlighted_id) = highlighted_subexpr {
        render_expression_text_recursive(&pool_ref, expr_id, highlighted_id)
    } else {
        rsx! {
            span { "{pool_ref.display_with_children(expr_id)}" }
        }
    }
}

fn render_expression_text_recursive(
    pool: &Pool,
    expr_id: ExprId,
    highlighted_id: ExprId,
) -> Element {
    let expr = &pool[expr_id];
    let is_highlighted = expr_id == highlighted_id;

    let content = match expr {
        ExprNode::Number(n) => rsx! { "{n}" },
        ExprNode::Variable(name_id) => rsx! { "{pool.display_name(*name_id)}" },
        ExprNode::Call { fun, .. } => {
            let children: Vec<_> = pool.children(expr_id).collect();
            let func_name = pool.display_function(*fun);

            if children.len() == 2 && is_binary_operator(&func_name) {
                rsx! {
                    "("
                    {render_expression_text_recursive(pool, children[1], highlighted_id)}
                    " {func_name} "
                    {render_expression_text_recursive(pool, children[0], highlighted_id)}
                    ")"
                }
            } else {
                rsx! {
                    "{func_name}("
                    for (i, child_id) in children.iter().enumerate().rev() {
                        if i < children.len() - 1 { ", " }
                        {render_expression_text_recursive(pool, *child_id, highlighted_id)}
                    }
                    ")"
                }
            }
        }
        ExprNode::Struct { name, .. } => {
            let children: Vec<_> = pool.children(expr_id).collect();
            let struct_name = pool.display_name(*name);

            rsx! {
                "{struct_name} {{ "
                for (i, child_id) in children.iter().enumerate().rev() {
                    if i < children.len() - 1 { ", " }
                    {render_expression_text_recursive(pool, *child_id, highlighted_id)}
                }
                " }}"
            }
        }
    };

    if is_highlighted {
        rsx! {
            span { class: "bg-yellow-200 px-1 rounded font-semibold", {content} }
        }
    } else {
        content
    }
}

fn is_binary_operator(op: &str) -> bool {
    matches!(
        op,
        "+" | "-" | "*" | "/" | "^" | "=" | "<" | ">" | "<=" | ">=" | "!=" | "&&" | "||"
    )
}


fn calculate_expression_depth(pool: &Pool, expr_id: ExprId) -> usize {
    let children: Vec<_> = pool.children(expr_id).collect();
    if children.is_empty() {
        1
    } else {
        1 + children
            .iter()
            .map(|&child| calculate_expression_depth(pool, child))
            .max()
            .unwrap_or(0)
    }
}

fn calculate_expression_size(pool: &Pool, expr_id: ExprId) -> usize {
    1 + pool
        .children(expr_id)
        .map(|child| calculate_expression_size(pool, child))
        .sum::<usize>()
}

#[component]
pub fn ExpressionDisplay(pool: Signal<Pool>, expr_id: ExprId, highlight: Option<bool>) -> Element {
    let pool_ref = pool.read();
    let expr = &pool_ref[expr_id];
    let is_highlighted = highlight.unwrap_or(false);

    let container_class = if is_highlighted {
        format!("{} bg-yellow-100 border-yellow-300", styles::CARD)
    } else {
        styles::CARD.to_string()
    };

    rsx! {
        div { class: container_class,
            div { class: styles::FLEX_BETWEEN,
                div { class: styles::FLEX_COL,
                    div { class: styles::FLEX_ROW,
                        span { class: "px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded font-medium",
                            "{get_expr_type_name(expr)}"
                        }
                        span { class: styles::TEXT_TINY, "ID: {expr_id:?}" }
                    }

                    div { class: styles::TEXT_MONO_LG,
                        "{pool_ref.display_with_children(expr_id)}"
                    }

                    ExpressionDetails { pool: pool, expr_id: expr_id }
                }

                div { class: styles::TEXT_TINY,
                    "Children: {pool_ref.children(expr_id).count()}"
                }
            }
        }
    }
}

#[component]
fn ExpressionDetails(pool: Signal<Pool>, expr_id: ExprId) -> Element {
    let pool_ref = pool.read();
    let expr = &pool_ref[expr_id];

    rsx! {
        div { class: styles::SPACE_Y_1,
            match expr {
                ExprNode::Number(n) => rsx! {
                    div { class: styles::TEXT_SMALL,
                        "Value: {n}"
                    }
                },
                ExprNode::Variable(name_id) => rsx! {
                    div { class: styles::TEXT_SMALL,
                        "Name: {pool_ref.display_name(*name_id)}"
                    }
                },
                ExprNode::Call { fun, arity, .. } => rsx! {
                    div { class: styles::TEXT_SMALL,
                        "Function: {pool_ref.display_function(*fun)}, Arity: {arity}"
                    }
                },
                ExprNode::Struct { name, arity, .. } => rsx! {
                    div { class: styles::TEXT_SMALL,
                        "Struct: {pool_ref.display_name(*name)}, Fields: {arity}"
                    }
                },
            }

            {
                let children: Vec<_> = pool_ref.children(expr_id).collect();
                if !children.is_empty() {
                    rsx! {
                        div { class: styles::TEXT_SMALL,
                            "Children: "
                            for (i, child_id) in children.iter().enumerate() {
                                if i > 0 { ", " }
                                span { class: "text-blue-600 cursor-pointer hover:underline",
                                    "{child_id:?}"
                                }
                            }
                        }
                    }
                } else {
                    rsx! { div {} }
                }
            }
        }
    }
}

#[component]
pub fn PatternDisplay(
    pool: Signal<Pool>,
    pattern_id: PatternId,
    highlight: Option<bool>,
) -> Element {
    let pool_ref = pool.read();
    let pattern = &pool_ref[pattern_id];
    let is_highlighted = highlight.unwrap_or(false);

    let container_class = if is_highlighted {
        format!("{} bg-green-100 border-green-300", styles::CARD)
    } else {
        styles::CARD.to_string()
    };

    rsx! {
        div { class: container_class,
            div { class: styles::FLEX_BETWEEN,
                div { class: styles::FLEX_COL,
                    div { class: styles::FLEX_ROW,
                        span { class: "px-2 py-1 text-xs bg-green-100 text-green-800 rounded font-medium",
                            "{get_pattern_type_name(pattern)}"
                        }
                        span { class: styles::TEXT_TINY, "ID: {pattern_id:?}" }
                    }

                    PatternDetails { pool: pool, pattern: *pattern }
                }
            }
        }
    }
}

#[component]
fn PatternDetails(pool: Signal<Pool>, pattern: Pattern) -> Element {
    let pool_ref = pool.read();

    rsx! {
        div { class: styles::SPACE_Y_1,
            match pattern {
                Pattern::Number(n) => rsx! {
                    div { class: styles::TEXT_MONO, "Number: {n}" }
                },
                Pattern::Variable(name_id) => rsx! {
                    div { class: styles::TEXT_MONO, "Variable: {pool_ref.display_name(name_id)}" }
                },
                Pattern::AnyNumber(name_id) => rsx! {
                    div { class: styles::TEXT_MONO, "AnyNumber: #{pool_ref.display_name(name_id)}" }
                },
                Pattern::Wildcard(name_id) => rsx! {
                    div { class: styles::TEXT_MONO, "Wildcard: ?{pool_ref.display_name(name_id)}" }
                },
                Pattern::Call { fun, arity, .. } => rsx! {
                    div { class: styles::TEXT_MONO,
                        "Call: {pool_ref.display_function(fun)}({arity} args)"
                    }
                },
                Pattern::Struct { name, arity, .. } => rsx! {
                    div { class: styles::TEXT_MONO,
                        "Struct: {pool_ref.display_name(name)}({arity} fields)"
                    }
                },
                Pattern::VarCallName { var, arity, .. } => rsx! {
                    div { class: styles::TEXT_MONO,
                        "VarCall: {pool_ref.display_name(var)}({arity} args)"
                    }
                },
                Pattern::VarStructName { var, arity, .. } => rsx! {
                    div { class: styles::TEXT_MONO,
                        "VarStruct: {pool_ref.display_name(var)}({arity} fields)"
                    }
                },
            }
        }
    }
}

#[component]
pub fn ActionDisplay(pool: Signal<Pool>, action_id: ActionId, highlight: Option<bool>) -> Element {
    let pool_ref = pool.read();
    let action = &pool_ref[action_id];
    let is_highlighted = highlight.unwrap_or(false);

    let container_class = if is_highlighted {
        format!("{} bg-purple-100 border-purple-300", styles::CARD)
    } else {
        styles::CARD.to_string()
    };

    rsx! {
        div { class: container_class,
            div { class: styles::FLEX_BETWEEN,
                div { class: styles::FLEX_COL,
                    div { class: styles::FLEX_ROW,
                        span { class: "px-2 py-1 text-xs bg-purple-100 text-purple-800 rounded font-medium",
                            "{get_action_type_name(action)}"
                        }
                        span { class: styles::TEXT_TINY, "ID: {action_id:?}" }
                    }

                    ActionDetails { pool: pool, action: *action }
                }
            }
        }
    }
}

#[component]
fn ActionDetails(pool: Signal<Pool>, action: Action) -> Element {
    let pool_ref = pool.read();

    rsx! {
        div { class: styles::SPACE_Y_1,
            match action {
                Action::Number(n) => rsx! {
                    div { class: styles::TEXT_MONO, "Number: {n}" }
                },
                Action::Variable(name_id) => rsx! {
                    div { class: styles::TEXT_MONO, "Variable: {pool_ref.display_name(name_id)}" }
                },
                Action::Call { fun, arity, .. } => rsx! {
                    div { class: styles::TEXT_MONO,
                        "Call: {pool_ref.display_function(fun)}({arity} args)"
                    }
                },
                Action::Struct { name, arity, .. } => rsx! {
                    div { class: styles::TEXT_MONO,
                        "Struct: {pool_ref.display_name(name)}({arity} fields)"
                    }
                },
                Action::Compute { arity, .. } => rsx! {
                    div { class: styles::TEXT_MONO,
                        "Compute: {arity} operands"
                    }
                },
                Action::VarCallName { var, arity, .. } => rsx! {
                    div { class: styles::TEXT_MONO,
                        "VarCall: {pool_ref.display_name(var)}({arity} args)"
                    }
                },
                Action::VarStructName { var, arity, .. } => rsx! {
                    div { class: styles::TEXT_MONO,
                        "VarStruct: {pool_ref.display_name(var)}({arity} fields)"
                    }
                },
            }
        }
    }
}

#[component]
pub fn RuleDisplay(pool: Signal<Pool>, rule_id: RuleId, highlight: Option<bool>) -> Element {
    let pool_ref = pool.read();
    let rule = &pool_ref[rule_id];
    let is_highlighted = highlight.unwrap_or(false);

    let container_class = if is_highlighted {
        format!("{} bg-orange-100 border-orange-300", styles::CARD)
    } else {
        styles::CARD.to_string()
    };

    rsx! {
        div { class: container_class,
            div { class: styles::FLEX_COL,
                div { class: styles::FLEX_BETWEEN,
                    div { class: styles::FLEX_ROW,
                        span { class: "px-2 py-1 text-xs bg-orange-100 text-orange-800 rounded font-medium",
                            "Rule"
                        }
                        span { class: styles::TEXT_TINY, "ID: {rule_id:?}" }
                    }
                    span { class: styles::LABEL_INLINE, "{pool_ref.display_name(rule.name)}" }
                }

                div { class: styles::FLEX_ROW,
                    div { class: "flex-1",
                        PatternDisplay { pool: pool, pattern_id: rule.pattern, highlight: false }
                    }
                    div { class: "flex-none px-4 flex items-center",
                        span { class: "text-2xl text-gray-400", "⇒" }
                    }
                    div { class: "flex-1",
                        ActionDisplay { pool: pool, action_id: rule.action, highlight: false }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PrebuiltRulesets(on_ruleset_selected: EventHandler<String>) -> Element {
    rsx! {
        div { class: "space-y-2",
            label { class: "block text-sm font-medium text-gray-700", "Prebuilt Rulesets" }
            div { class: "flex flex-wrap gap-2",
                button {
                    class: "px-3 py-1 text-xs bg-purple-100 text-purple-700 rounded hover:bg-purple-200 transition-colors",
                    onclick: move |_| {
                        on_ruleset_selected.call(get_basic_arithmetic_ruleset());
                    },
                    "Basic Arithmetic"
                }
                button {
                    class: "px-3 py-1 text-xs bg-green-100 text-green-700 rounded hover:bg-green-200 transition-colors",
                    onclick: move |_| {
                        on_ruleset_selected.call(get_monoid_ruleset());
                    },
                    "Monoid Laws"
                }
                button {
                    class: "px-3 py-1 text-xs bg-blue-100 text-blue-700 rounded hover:bg-blue-200 transition-colors",
                    onclick: move |_| {
                        on_ruleset_selected.call(get_group_ruleset());
                    },
                    "Group Laws"
                }
                button {
                    class: "px-3 py-1 text-xs bg-yellow-100 text-yellow-700 rounded hover:bg-yellow-200 transition-colors",
                    onclick: move |_| {
                        on_ruleset_selected.call(get_semiring_ruleset());
                    },
                    "Semiring Laws"
                }
                button {
                    class: "px-3 py-1 text-xs bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors",
                    onclick: move |_| {
                        on_ruleset_selected.call(String::new());
                    },
                    "Clear"
                }
            }
        }
    }
}

fn get_basic_arithmetic_ruleset() -> String {
    r#"basic_arithmetic {

  add_coeffs        : #a * ?x  + #b * ?x => [a + b] * x
  sub_coeffs        : #a * ?x  - #b * ?x => [a - b] * x
  mul_coeffs        : #a * ( #b * ?x )   => [a * b] * x
  div_              : #a * ?x  / #a * ?y =>  x / y

  add_numbers       : #a + #b            => [a + b]
  sub_numbers       : #a - #b            => [a - b]
  mul_numbers       : #a * #b            => [a * b]
  div_numbers       : #a / #b            => [a / b]

  double            : ?x + ?x            => 2 * x
  incr_coeff        : #a * ?x + ?x       => [a+1] * x
  add_assoc      : (?x + ?y) + ?z => x + (y + z)
  add_comm       : ?x + ?y        => y + x
  add_zero_left  : 0 + ?x         => x
  add_zero_right : ?x + 0         => x
  mul_assoc      : (?x * ?y) * ?z => x * (y * z)
  mul_comm       : ?x * ?y        => y * x
  mul_one_left   : 1 * ?x         => x
  mul_one_right  : ?x * 1         => x
  mul_zero_left  : 0 * ?x         => 0
  mul_zero_right : ?x * 0         => 0
  left_distrib   : ?x * (?y + ?z) => x * y + x * z
  right_distrib  : (?x + ?y) * ?z => x * z + y * z

  b_add_assoc      : ?x + (?y + ?z) => (x + y) + z
  b_add_zero_left  : ?x =>0 + x
  b_add_zero_right : ?x => x + 0
  b_mul_assoc      : ?x * (?y * ?z) => (x * y) * z
  b_mul_one_left   : ?x => 1 * x
  b_mul_one_right  : ?x => x * 1
  b_mul_zero_left  : 0  => 0 * new
  b_mul_zero_right : 0  => new * 0
  b_left_distrib   : ?x * ?y + ?x * ?z  => x * (y + z)
  b_right_distrib  :  ?x * ?z + ?y * ?z => (x + y) * z


}"#
    .to_string()
}

fn get_monoid_ruleset() -> String {
    r#"monoid {
  left_identity  : 0 + ?x         => x
  right_identity : ?x + 0         => x
  associativity  : (?x + ?y) + ?z => x + (y + z)

 b_left_identity  : ?x => 0 + x
 b_right_identity : ?x => x + 0
 b_associativity  : ?x + (?y + ?z) => (x + y ) + z
}"#
    .to_string()
}

fn get_group_ruleset() -> String {
    r#"group {
  left_identity      : 0 + ?x         => x
  right_identity     : ?x + 0         => x
  left_inverse       : -?x + ?x       => 0
  right_inverse      : ?x + -?x       => 0
  associativity      : (?x + ?y) + ?z => x + (y + z)
  inverse_inverse    : -(-(?x))       => x
  inverse_identity   : -(0)           => 0

  b_left_identity    : ?x             => 0 + x
  b_right_identity   : ?x             => x + 0
  b_left_inverse     : 0              => -x + x
  b_right_inverse    : 0              => x + -x
  b_associativity    : ?x + (?y + ?z) => (x + y) + z
  b_inverse_inverse  : ?x             => -(-(x))
  b_inverse_identity : 0              => -(0)



}"#
    .to_string()
}

fn get_semiring_ruleset() -> String {
    r#"semiring {
  add_assoc      : (?x + ?y) + ?z => x + (y + z)
  add_comm       : ?x + ?y        => y + x
  add_zero_left  : 0 + ?x         => x
  add_zero_right : ?x + 0         => x
  mul_assoc      : (?x * ?y) * ?z => x * (y * z)
  mul_one_left   : 1 * ?x         => x
  mul_one_right  : ?x * 1         => x
  mul_zero_left  : 0 * ?x         => 0
  mul_zero_right : ?x * 0         => 0
  left_distrib   : ?x * (?y + ?z) => x * y + x * z
  right_distrib  : (?x + ?y) * ?z => x * z + y * z

  b_add_assoc      : ?x + (?y + ?z) => (x + y) + z
  b_add_zero_left  : ?x =>0 + x
  b_add_zero_right : ?x => x + 0
  b_mul_assoc      : ?x * (?y * ?z) => (x * y) * z
  b_mul_one_left   : ?x => 1 * x
  b_mul_one_right  : ?x => x * 1
  b_mul_zero_left  : 0  => 0 * new
  b_mul_zero_right : 0  => new * 0
  b_left_distrib   : ?x * ?y + ?x * ?z  => x * (y + z)
  b_right_distrib  :  ?x * ?z + ?y * ?z => (x + y) * z


}"#
    .to_string()
}

#[component]
pub fn ParsedRulesetsList(
    pool: Signal<Pool>,
    parsed_rulesets: Vec<RulesetId>,
    selected_ruleset: Option<RulesetId>,
    on_ruleset_selected: EventHandler<RulesetId>,
) -> Element {
    let pool_ref = pool.read();
    
    if parsed_rulesets.is_empty() {
        return rsx! {
            div { class: "text-sm text-gray-500 italic p-4",
                "No rulesets parsed yet. Parse a ruleset above to see it here."
            }
        };
    }
    
    rsx! {
        div { class: "space-y-2",
            h3 { class: "text-sm font-medium text-gray-700", "Parsed Rulesets" }
            div { class: "space-y-2 max-h-48 overflow-y-auto",
                for ruleset_id in parsed_rulesets {
                    div {
                        key: "{ruleset_id:?}",
                        class: if selected_ruleset == Some(ruleset_id) {
                            "p-3 bg-blue-100 border-2 border-blue-400 rounded-lg cursor-pointer transition-colors"
                        } else {
                            "p-3 bg-gray-50 border-2 border-gray-200 rounded-lg cursor-pointer hover:bg-gray-100 transition-colors"
                        },
                        onclick: move |_| {
                            on_ruleset_selected.call(ruleset_id);
                        },
                        
                        div { class: "flex items-center justify-between",
                            div { class: "flex items-center gap-3",
                                span { class: "font-medium text-gray-800",
                                    "{pool_ref.display_name(pool_ref[ruleset_id].name)}"
                                }
                                span { class: "text-xs text-gray-500",
                                    "({pool_ref.get_ruleset_rule_count(ruleset_id)} rules)"
                                }
                            }
                            if selected_ruleset == Some(ruleset_id) {
                                span { class: "text-sm text-blue-600 font-medium", "✓ Selected" }
                            }
                        }
                        
                        if selected_ruleset == Some(ruleset_id) {
                            div { class: "mt-2 text-xs text-gray-600",
                                "Rules: ",
                                {
                                    let rules: Vec<_> = pool_ref.get_ruleset_rules(ruleset_id).take(5).collect();
                                    let total_count = pool_ref.get_ruleset_rule_count(ruleset_id);
                                    rsx! {
                                        for (i, rule) in rules.iter().enumerate() {
                                            if i > 0 { ", " }
                                            span { class: "font-mono",
                                                "{pool_ref.display_name(rule.name)}"
                                            }
                                        }
                                        if total_count > 5 {
                                            span { ", ..." }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn get_expr_type_name(expr: &ExprNode) -> &'static str {
    match expr {
        ExprNode::Number(_) => "Number",
        ExprNode::Variable(_) => "Variable",
        ExprNode::Call { .. } => "Call",
        ExprNode::Struct { .. } => "Struct",
    }
}

fn get_pattern_type_name(pattern: &Pattern) -> &'static str {
    match pattern {
        Pattern::Number(_) => "Number",
        Pattern::Variable(_) => "Variable",
        Pattern::AnyNumber(_) => "AnyNumber",
        Pattern::Wildcard(_) => "Wildcard",
        Pattern::Call { .. } => "Call",
        Pattern::Struct { .. } => "Struct",
        Pattern::VarCallName { .. } => "VarCall",
        Pattern::VarStructName { .. } => "VarStruct",
    }
}

fn get_action_type_name(action: &Action) -> &'static str {
    match action {
        Action::Number(_) => "Number",
        Action::Variable(_) => "Variable",
        Action::Call { .. } => "Call",
        Action::Struct { .. } => "Struct",
        Action::Compute { .. } => "Compute",
        Action::VarCallName { .. } => "VarCall",
        Action::VarStructName { .. } => "VarStruct",
    }
}
