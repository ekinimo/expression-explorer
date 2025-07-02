use super::super::{
    display_components::CompactExpressionCard,
    primitives::TransformationGraph,
};
use crate::{ActionId, Children, DisplayNode, ExprId, PatternId, Pool, rules::Match, search::{SearchEngine, SearchConfig, SearchPath}};
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

    let mut highlighted_subexpr = use_signal(|| None::<ExprId>);
    let mut current_matches = use_signal(Vec::<Match>::new);
    let mut rules_panel_collapsed = use_signal(|| false);
    let mut hovered_rule_index = use_signal(|| None::<usize>);
    let _show_search_panel = use_signal(|| false);

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

                        // button {
                        //     class: format!("px-3 py-2 rounded transition-colors text-sm {}",
                        //         if *show_search_panel.read() {
                        //             "bg-green-100 text-green-700"
                        //         } else {
                        //             "bg-gray-100 text-gray-700"
                        //         }
                        //     ),
                        //     onclick: move |_| {
                        //         let current = *show_search_panel.read();
                        //         show_search_panel.set(!current);
                        //     },
                        //     if *show_search_panel.read() { "Hide Search" } else { "Path Search" }
                        // }

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
                            highlighted_subexpr: *highlighted_subexpr.read(),
                        }
                    }
                    
                    // if *show_search_panel.read() {
                    //     SearchPanel {
                    //         pool: pool,
                    //         current_expr: *expr_id,
                    //         on_select_expr: move |expr| {
                    //             current_expr.set(Some(expr));
                    //         }
                    //     }
                    // }
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
                    div { class: "flex-1 bg-white overflow-hidden",
                        style: "max-width: calc(100% - 320px);", // Ensure space for rules panel
                        TransformationGraph {
                            pool: pool,
                            current_expr: Some(*expr_id),
                            on_node_click: Some(EventHandler::new(move |clicked_expr: ExprId| {
                                current_expr.set(Some(clicked_expr));
                            })),
                        }
                    }

                    if !*rules_panel_collapsed.read() {
                        div { class: "w-80 flex-shrink-0 bg-white shadow-lg border-l border-gray-200 overflow-y-auto",
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
    let mut search_query = use_signal(String::new);
    
    rsx! {
        div { class: "p-4",
            h2 { class: "text-lg font-semibold mb-4", "Applicable Rules" }
            
            if !matches.is_empty() {
                div { class: "mb-4",
                    input {
                        class: "w-full px-3 py-2 border border-gray-300 rounded text-sm placeholder-gray-400",
                        placeholder: "Search rules by name...",
                        value: "{search_query}",
                        oninput: move |evt| {
                            search_query.set(evt.value());
                        }
                    }
                }
            }

            if matches.is_empty() {
                div { class: "text-gray-500 text-center py-8",
                    "No rules match this expression."
                    br {}
                    "Parse a ruleset to see applicable transformations."
                }
            } else {
                div { class: "space-y-3",
                    {
                        let query = search_query.read().to_lowercase();
                        let filtered_matches: Vec<_> = matches.iter().enumerate()
                            .filter(|(_, match_)| {
                                if query.is_empty() {
                                    true
                                } else {
                                    let pool_ref = pool.read();
                                    let rule = pool_ref[match_.rule_id];
                                    pool_ref.display_name(rule.name).to_lowercase().contains(&query)
                                }
                            })
                            .collect();
                        
                        if filtered_matches.is_empty() {
                            rsx! {
                                div { class: "text-gray-500 text-center py-4",
                                    "No rules match \"{query}\""
                                }
                            }
                        } else {
                            rsx! {
                                for (idx, match_) in filtered_matches {
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

#[component]
fn SearchPanel(
    pool: Signal<Pool>,
    current_expr: ExprId,
    on_select_expr: EventHandler<ExprId>,
) -> Element {
    let mut source_expr = use_signal(|| current_expr);
    let mut target_expr_text = use_signal(String::new);
    let mut target_expr_error = use_signal(|| None::<String>);
    let mut search_strategy = use_signal(|| "bfs".to_string());
    let mut max_depth = use_signal(|| 10);
    let mut search_results = use_signal(|| None::<Vec<SearchPath>>);
    let mut is_searching = use_signal(|| false);
    
    rsx! {
        div { class: "p-4 bg-gray-50 border-b",
            h3 { class: "text-lg font-semibold mb-4", "Path Search" }
            
            div { class: "grid grid-cols-2 gap-4 mb-4",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "Source Expression" }
                    select {
                        class: "w-full px-3 py-2 border border-gray-300 rounded text-sm",
                        value: "{(*source_expr.read()).0}",
                        onchange: move |evt| {
                            if let Ok(idx) = evt.value().parse::<usize>() {
                                source_expr.set(ExprId(idx));
                            }
                        },
                        
                        {
                            let current_source = *source_expr.read();
                            let pool_ref = pool.read();
                            rsx! {
                                for (idx, _) in pool_ref.exprs.iter().enumerate() {
                                    option {
                                        key: "{idx}",
                                        value: "{idx}",
                                        selected: ExprId(idx) == current_source,
                                        "{ExprId(idx):?}: {pool_ref.display_with_children(ExprId(idx))}"
                                    }
                                }
                            }
                        }
                    }
                }
                
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "Target Expression (optional)" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 border border-gray-300 rounded text-sm",
                        placeholder: "e.g., x + 1, sin(x), etc. (leave empty for any)",
                        value: "{target_expr_text}",
                        oninput: move |evt| {
                            target_expr_text.set(evt.value());
                            target_expr_error.set(None);
                        }
                    }
                    if let Some(error) = target_expr_error.read().as_ref() {
                        div { class: "text-red-500 text-xs mt-1", "{error}" }
                    }
                }
            }
            
            div { class: "grid grid-cols-2 gap-4 mb-4",
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "Search Strategy" }
                    select {
                        class: "w-full px-3 py-2 border border-gray-300 rounded text-sm",
                        value: "{search_strategy}",
                        onchange: move |evt| {
                            search_strategy.set(evt.value());
                        },
                        
                        option { value: "bfs", "Breadth-First Search" }
                        option { value: "dijkstra", "Dijkstra (Cost-based)" }
                        option { value: "beam", "Beam Search" }
                        option { value: "random", "Random Search" }
                    }
                }
                
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "Max Depth" }
                    input {
                        r#type: "number",
                        class: "w-full px-3 py-2 border border-gray-300 rounded text-sm",
                        value: "{max_depth}",
                        min: "1",
                        max: "50",
                        oninput: move |evt| {
                            if let Ok(depth) = evt.value().parse::<usize>() {
                                max_depth.set(depth);
                            }
                        }
                    }
                }
            }
            
            div { class: "flex gap-2 mb-4",
                button {
                    class: "px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400",
                    disabled: *is_searching.read(),
                    onclick: move |_| {
                        is_searching.set(true);
                        search_results.set(None);
                        target_expr_error.set(None);
                        
                        // Read all state before borrowing pool
                        let source = *source_expr.read();
                        let target_text = target_expr_text.read().clone();
                        let strategy = search_strategy.read().clone();
                        let depth = *max_depth.read();
                        
                        // Ensure source expression has equivalence group
                        let mut pool_write = pool.write();
                        pool_write.update_equivalence_groups(source);
                        drop(pool_write);
                        
                        // Parse target expression if provided
                        let mut pool_write = pool.write();
                        let target = if target_text.trim().is_empty() {
                            None
                        } else {
                            match crate::parser::expr::parse_expression(&target_text, &mut pool_write) {
                                Ok(expr_id) => {
                                    // Ensure the newly parsed expression has equivalence groups set up
                                    pool_write.update_equivalence_groups(expr_id);
                                    Some(expr_id)
                                },
                                Err(e) => {
                                    target_expr_error.set(Some(format!("Parse error: {}", e)));
                                    is_searching.set(false);
                                    return;
                                }
                            }
                        };
                        drop(pool_write);
                        
                        let mut pool_write = pool.write();
                        let mut config = SearchConfig::default();
                        config.max_depth = depth;
                        
                        let mut engine = SearchEngine::new(config);
                        let paths = match strategy.as_str() {
                            "bfs" => engine.bounded_bfs(&mut *pool_write, source, target),
                            "dijkstra" => engine.bounded_dijkstra(
                                &mut *pool_write,
                                source, 
                                target,
                                |_, _, _, _| 1.0
                            ),
                            "beam" => {
                                engine.beam_search(&mut *pool_write, source, |pool, expr, _path| {
                                    if let Some(t) = target {
                                        if pool.expr_eq(expr, t) { 1000.0 } else { 1.0 }
                                    } else {
                                        1.0
                                    }
                                })
                            },
                            "random" => engine.random_search(&mut *pool_write, source, 10),
                            _ => vec![],
                        };
                        drop(pool_write);
                        
                        search_results.set(Some(paths));
                        is_searching.set(false);
                    },
                    
                    if *is_searching.read() { "Searching..." } else { "Search" }
                }
                
                if search_results.read().is_some() {
                    button {
                        class: "px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300",
                        onclick: move |_| {
                            search_results.set(None);
                        },
                        "Clear Results"
                    }
                }
            }
            
            if let Some(paths) = search_results.read().as_ref() {
                div { class: "border-t pt-4",
                    if paths.is_empty() {
                        div { class: "text-gray-500 text-center py-4",
                            "No paths found"
                        }
                    } else {
                        div { class: "space-y-3",
                            h4 { class: "font-medium mb-2", "Found {paths.len()} path(s):" }
                            
                            for (idx, path) in paths.iter().enumerate().take(10) {
                                SearchResultItem {
                                    key: "{idx}",
                                    pool: pool,
                                    path: path.clone(),
                                    index: idx,
                                    on_select: move |expr| {
                                        on_select_expr.call(expr);
                                    }
                                }
                            }
                            
                            if paths.len() > 10 {
                                div { class: "text-sm text-gray-500 text-center mt-2",
                                    "Showing first 10 of {paths.len()} paths"
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
fn SearchResultItem(
    pool: Signal<Pool>,
    path: SearchPath,
    index: usize,
    on_select: EventHandler<ExprId>,
) -> Element {
    let pool_ref = pool.read();
    
    rsx! {
        div { class: "bg-white border rounded p-3 hover:shadow-md transition-shadow",
            div { class: "flex items-center justify-between mb-2",
                span { class: "font-medium text-sm", "Path {index + 1}" }
                span { class: "text-xs text-gray-500", "Length: {path.length}, Cost: {path.cost:.2}" }
            }
            
            if path.steps.is_empty() {
                div { class: "text-sm text-gray-600", "Direct match" }
            } else {
                div { class: "space-y-1",
                    for (idx, &(from, rule, to)) in path.steps.iter().enumerate() {
                        div { 
                            key: "{idx}",
                            class: "text-xs flex items-center gap-2",
                            
                            span { 
                                class: "cursor-pointer hover:text-blue-600",
                                onclick: move |_| on_select.call(from),
                                "{pool_ref.display_with_children(from)}"
                            }
                            
                            span { class: "text-gray-400", "→" }
                            
                            span { class: "font-medium text-purple-600",
                                "{pool_ref.display_name(pool_ref[rule].name)}"
                            }
                            
                            span { class: "text-gray-400", "→" }
                            
                            span { 
                                class: "cursor-pointer hover:text-blue-600",
                                onclick: move |_| on_select.call(to),
                                "{pool_ref.display_with_children(to)}"
                            }
                        }
                    }
                }
            }
        }
    }
}
