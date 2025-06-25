use super::super::{
    display_components::{CompactExpressionCard, ViewMode},
    primitives::TransformationGraph,
};
use crate::{ActionId, Children, DisplayNode, ExprId, PatternId, Pool, rules::Match};
use dioxus::prelude::*;

#[component]
pub fn ExplorerPage(pool: Signal<Pool>) -> Element {
    let mut current_expr = use_signal(|| {
        let pool_ref = pool.read();
        if !pool_ref.exprs.is_empty() {
            Some(ExprId(pool_ref.exprs.len() - 1))
        } else {
            None
        }
    });

    let mut view_mode = use_signal(|| ViewMode::Text);
    let mut highlighted_subexpr = use_signal(|| None::<ExprId>);
    let mut current_matches = use_signal(Vec::<Match>::new);
    let mut rules_panel_collapsed = use_signal(|| false);
    let mut hovered_rule_index = use_signal(|| None::<usize>);

    use_effect(move || {
        if let Some(expr_id) = current_expr.read().as_ref() {
            let matches = pool.read().find_matches(*expr_id);
            current_matches.set(matches);
        } else {
            current_matches.set(Vec::new());
        }
    });

    use_context_provider(|| current_expr);

    rsx! {
        div { class: "h-screen flex flex-col",
            if let Some(expr_id) = current_expr.read().as_ref() {
                div { class: "flex-shrink-0 bg-gray-50 border-b border-gray-200",
                    div { class: "flex items-center justify-between p-4 border-b border-gray-100",
                        div { class: "flex items-center gap-4",
                            h1 { class: "text-xl font-bold text-gray-800", "Expression Explorer" }
                            ExpressionQuickSelector {
                                pool: pool,
                                current_expr: *expr_id,
                                on_change: move |new_expr| {
                                    current_expr.set(Some(new_expr));
                                }
                            }
                        }

                        button {
                            class: "px-3 py-2 rounded transition-colors text-sm bg-red-100 text-red-700 hover:bg-red-200",
                            onclick: move |_| {
                                pool.write().reset();
                                current_expr.set(None);
                            },
                            "Reset Pool"
                        }

                        button {
                            class: format!("px-3 py-2 rounded transition-colors text-sm {}",
                                if *rules_panel_collapsed.read() {
                                    "bg-blue-100 text-blue-700"
                                } else {
                                    "bg-gray-100 text-gray-700"
                                }
                            ),
                            onclick: move |_| {
                                let current_collapsed = *rules_panel_collapsed.read();
                                rules_panel_collapsed.set(!current_collapsed);
                            },
                            if *rules_panel_collapsed.read() { "Show Rules" } else { "Hide Rules" }
                        }
                    }

                    div { class: "p-4",
                        CompactExpressionCard {
                            pool: pool,
                            expr_id: *expr_id,
                            view_mode: *view_mode.read(),
                            highlighted_subexpr: *highlighted_subexpr.read(),
                            on_view_mode_change: move |new_mode| {
                                view_mode.set(new_mode);
                            },
                        }
                    }
                }
            } else {
                div { class: "flex-shrink-0 bg-white border-b border-gray-200 p-8 text-center",
                    h1 { class: "text-xl font-bold text-gray-800 mb-4", "Expression Explorer" }
                    p { class: "text-gray-600 mb-4", "Select an expression to explore:" }
                    ExpressionSelector {
                        pool: pool,
                        on_select: move |expr_id| {
                            current_expr.set(Some(expr_id));
                        }
                    }
                }
            }

            if let Some(expr_id) = current_expr.read().as_ref() {
                div { class: "flex-1 flex min-h-0",
                    div { class: "flex-1 bg-white",
                        TransformationGraph {
                            pool: pool,
                            current_expr: Some(*expr_id),
                            on_node_click: Some(EventHandler::new(move |clicked_expr: ExprId| {
                                current_expr.set(Some(clicked_expr));
                            })),
                        }
                    }

                    if !*rules_panel_collapsed.read() {
                        div { class: "w-80 bg-white shadow-lg border-l border-gray-200 overflow-y-auto",
                            RulesSidebar {
                                pool: pool,
                                matches: current_matches.read().clone(),
                                hovered_rule_index: hovered_rule_index,
                                on_hover_rule: move |index| {
                                    hovered_rule_index.set(index);
                                    if let Some(idx) = index {
                                        if let Some(match_) = current_matches.read().get(idx) {
                                            highlighted_subexpr.set(Some(match_.offset));
                                        }
                                    } else {
                                        highlighted_subexpr.set(None);
                                    }
                                },
                                on_apply_rule: move |match_| {
                                    let mut p = pool.write();
                                    if let Some(new_expr) = p.apply_rule(&match_) {
                                        current_expr.set(Some(new_expr));
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ExpressionSelector(pool: Signal<Pool>, on_select: EventHandler<ExprId>) -> Element {
    let pool_ref = pool.read();

    rsx! {
        div { class: "space-y-4",
            if pool_ref.exprs.is_empty() {
                div { class: "text-gray-500 text-center",
                    "No expressions available. Go to the Input page to parse some expressions first."
                }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 max-h-60 overflow-y-auto",
                    for (idx, _) in pool_ref.exprs.iter().enumerate() {
                        {
                            let expr_id = ExprId(idx);
                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: "p-3 border border-gray-200 rounded-lg hover:border-blue-300 hover:shadow-md transition-all cursor-pointer bg-white",
                                    onclick: move |_| {
                                        on_select.call(expr_id);
                                    },

                                    div { class: "space-y-2",
                                        div { class: "flex items-center gap-2",
                                            span { class: "px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded font-medium",
                                                "ID: {expr_id:?}"
                                            }
                                        }
                                        div { class: "font-mono text-sm text-gray-800",
                                            "{pool_ref.display_with_children(expr_id)}"
                                        }
                                        div { class: "text-xs text-gray-500",
                                            "Children: {pool_ref.children(expr_id).count()}"
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

#[component]
fn ExpressionQuickSelector(
    pool: Signal<Pool>,
    current_expr: ExprId,
    on_change: EventHandler<ExprId>,
) -> Element {
    let pool_ref = pool.read();

    rsx! {
        div { class: "flex items-center gap-2",
            span { class: "text-sm text-gray-600", "Expression:" }
            select {
                class: "px-3 py-1 border border-gray-300 rounded text-sm bg-white",
                value: "{current_expr.0}",
                onchange: move |evt| {
                    if let Ok(idx) = evt.value().parse::<usize>() {
                        on_change.call(ExprId(idx));
                    }
                },

                for (idx, _) in pool_ref.exprs.iter().enumerate() {
                    {
                        let expr_id = ExprId(idx);
                        rsx! {
                            option {
                                key: "{idx}",
                                value: "{idx}",
                                "{expr_id:?}: {pool_ref.display_with_children(expr_id)}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RulesSidebar(
    pool: Signal<Pool>,
    matches: Vec<Match>,
    hovered_rule_index: Signal<Option<usize>>,
    on_hover_rule: EventHandler<Option<usize>>,
    on_apply_rule: EventHandler<Match>,
) -> Element {
    rsx! {
        div { class: "p-4",
            h2 { class: "text-lg font-semibold mb-4", "Applicable Rules" }

            if matches.is_empty() {
                div { class: "text-gray-500 text-center py-8",
                    "No rules match this expression."
                    br {}
                    "Parse a ruleset to see applicable transformations."
                }
            } else {
                div { class: "space-y-3",
                    for (idx, match_) in matches.iter().enumerate() {
                        RuleMatchItem {
                            key: "{idx}",
                            pool: pool,
                            match_: match_.clone(),
                            index: idx,
                            is_hovered: *hovered_rule_index.read() == Some(idx),
                            on_hover: move |hovered| {
                                on_hover_rule.call(if hovered { Some(idx) } else { None });
                            },
                            on_apply: move |m| on_apply_rule.call(m),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RuleMatchItem(
    pool: Signal<Pool>,
    match_: Match,
    index: usize,
    is_hovered: bool,
    on_hover: EventHandler<bool>,
    on_apply: EventHandler<Match>,
) -> Element {
    let pool_ref = pool.read();
    let rule = pool_ref[match_.rule_id];

    rsx! {
        div {
            class: format!("p-3 border rounded-lg transition-all cursor-pointer {}",
                if is_hovered {
                    "border-yellow-300 bg-yellow-50"
                } else {
                    "border-gray-200 hover:border-blue-300 hover:shadow-sm"
                }
            ),
            onmouseenter: move |_| on_hover.call(true),
            onmouseleave: move |_| on_hover.call(false),

            div { class: "space-y-2",
                div { class: "flex items-center justify-between",
                    div { class: "text-sm font-medium text-gray-800",
                        "Rule: {pool_ref.display_name(rule.name)}"
                    }
                    button {
                        class: "px-2 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors",
                        onclick: move |_| on_apply.call(match_.clone()),
                        "Apply"
                    }
                }

                div { class: "text-xs text-gray-600 font-mono",
                    "Pattern: {pool_ref.display_with_children(PatternId(rule.pattern.0))}"
                    br {}
                    "Action: {pool_ref.display_with_children(ActionId(rule.action.0))}"
                }

                if !match_.captures.is_empty() {
                    div { class: "text-xs",
                        div { class: "text-gray-600 mb-1", "Captures:" }
                        div { class: "space-y-1",
                            for (name_id, captured_value) in &match_.captures {
                                div { class: "flex items-center gap-1",
                                    span { class: "text-purple-600 font-mono", "?{pool_ref[*name_id]}" }
                                    span { class: "text-gray-400", "→" }
                                    span { class: "text-gray-700 font-mono text-xs",
                                        match captured_value {
                                            crate::rules::CapturedValue::Expression(expr_id) => {
                                                pool_ref.display_with_children(*expr_id).to_string()
                                            }
                                            crate::rules::CapturedValue::Function(fun_id) => {
                                                format!("fn:{}", pool_ref[*fun_id])
                                            }
                                            crate::rules::CapturedValue::StructName(name_id) => {
                                                format!("struct:{}", pool_ref[*name_id])
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "text-xs text-gray-500",
                    "Matches at offset {match_.offset}"
                }
            }
        }
    }
}
