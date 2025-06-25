use super::{display_components::ViewMode, navigation::Page, primitives::UIError};
use crate::{ExprId, Pool, RuleId, RulesetId, rules::Match};
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct AppState {
    pub current_page: Page,
    pub navigation_history: Vec<Page>,
    pub loading_states: HashMap<String, bool>,
    pub error_states: HashMap<String, UIError>,
    pub search_query: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_page: Page::Input,
            navigation_history: vec![Page::Input],
            loading_states: HashMap::new(),
            error_states: HashMap::new(),
            search_query: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionState {
    pub current_expr: Option<ExprId>,
    pub selected_expressions: Vec<ExprId>,
    pub view_mode: ViewMode,
    pub highlighted_subexpr: Option<ExprId>,
}

impl Default for ExpressionState {
    fn default() -> Self {
        Self {
            current_expr: None,
            selected_expressions: Vec::new(),
            view_mode: ViewMode::Text,
            highlighted_subexpr: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RulesState {
    pub current_matches: Vec<Match>,
    pub applied_rules_history: Vec<(ExprId, RuleId, ExprId)>,
    pub hovered_rule_index: Option<usize>,
    pub rules_panel_collapsed: bool,
}

#[derive(Clone, Debug)]
pub struct InputState {
    pub expr_text: String,
    pub expr_error: Option<UIError>,
    pub expr_result: Option<ExprId>,
    pub ruleset_text: String,
    pub ruleset_error: Option<UIError>,
    pub ruleset_result: Option<RulesetId>,
    pub parsing_blocked: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            expr_text: "(x + y) * 2".to_string(),
            expr_error: None,
            expr_result: None,
            ruleset_text:
                "algebra {\n  double_add: ?x + ?x => 2 * x\n  identity_add: ?x + 0 => x\n}"
                    .to_string(),
            ruleset_error: None,
            ruleset_result: None,
            parsing_blocked: false,
        }
    }
}

#[component]
pub fn AppStateProvider(children: Element) -> Element {
    use_context_provider(|| Signal::new(AppState::default()));

    use_context_provider(|| Signal::new(Pool::new()));

    rsx! { {children} }
}

#[component]
pub fn ExpressionStateProvider(children: Element) -> Element {
    use_context_provider(|| Signal::new(ExpressionState::default()));
    use_context_provider(|| Signal::new(RulesState::default()));

    rsx! { {children} }
}

#[component]
pub fn InputStateProvider(children: Element) -> Element {
    use_context_provider(|| Signal::new(InputState::default()));

    rsx! { {children} }
}

pub fn use_app_state() -> Signal<AppState> {
    use_context::<Signal<AppState>>()
}

pub fn use_pool() -> Signal<Pool> {
    use_context::<Signal<Pool>>()
}

pub fn use_expression_state() -> Signal<ExpressionState> {
    use_context::<Signal<ExpressionState>>()
}

pub fn use_rules_state() -> Signal<RulesState> {
    use_context::<Signal<RulesState>>()
}

pub fn use_input_state() -> Signal<InputState> {
    use_context::<Signal<InputState>>()
}

pub fn use_app_navigation() -> (Page, Signal<AppState>) {
    let app_state = use_app_state();
    let current_page = app_state.read().current_page.clone();
    (current_page, app_state)
}

pub fn navigate_to(app_state: &mut AppState, page: Page) {
    app_state
        .navigation_history
        .push(app_state.current_page.clone());
    app_state.current_page = page;
}

pub fn navigate_back(app_state: &mut AppState) {
    if let Some(previous_page) = app_state.navigation_history.pop() {
        app_state.current_page = previous_page;
    }
}

pub fn use_current_expression() -> (
    Option<ExprId>,
    Signal<ExpressionState>,
    Signal<RulesState>,
    Signal<Pool>,
) {
    let expression_state = use_expression_state();
    let rules_state = use_rules_state();
    let pool = use_pool();
    let current_expr = expression_state.read().current_expr;
    (current_expr, expression_state, rules_state, pool)
}

pub fn set_current_expression(
    expression_state: &mut ExpressionState,
    rules_state: &mut RulesState,
    pool: &Pool,
    expr_id: Option<ExprId>,
) {
    expression_state.current_expr = expr_id;

    if let Some(expr_id) = expr_id {
        rules_state.current_matches = pool.find_matches(expr_id);
    } else {
        rules_state.current_matches.clear();
    }
}

pub fn use_view_mode() -> (ViewMode, Signal<ExpressionState>) {
    let expression_state = use_expression_state();
    let view_mode = expression_state.read().view_mode;
    (view_mode, expression_state)
}

pub fn use_rules_panel() -> (bool, Signal<RulesState>) {
    let rules_state = use_rules_state();
    let collapsed = rules_state.read().rules_panel_collapsed;
    (collapsed, rules_state)
}

pub fn use_rule_application() -> (Signal<ExpressionState>, Signal<RulesState>, Signal<Pool>) {
    let expression_state = use_expression_state();
    let rules_state = use_rules_state();
    let pool = use_pool();
    (expression_state, rules_state, pool)
}

pub fn apply_rule_at_index(
    expression_state: &mut ExpressionState,
    rules_state: &mut RulesState,
    pool: &mut Pool,
    match_index: usize,
) {
    if let (Some(match_info), Some(current_expr)) = (
        rules_state.current_matches.get(match_index).cloned(),
        expression_state.current_expr,
    ) {
        if let Some(result_expr) = pool.apply_rule(&match_info) {
            rules_state
                .applied_rules_history
                .push((current_expr, match_info.rule_id, result_expr));

            expression_state.current_expr = Some(result_expr);

            rules_state.current_matches = pool.find_matches(result_expr);
        }
    }
}

pub fn use_loading_error() -> Signal<AppState> {
    use_app_state()
}

pub fn is_loading(app_state: &AppState, key: &str) -> bool {
    app_state.loading_states.get(key).copied().unwrap_or(false)
}

pub fn set_loading(app_state: &mut AppState, key: String, loading: bool) {
    if loading {
        app_state.loading_states.insert(key, true);
    } else {
        app_state.loading_states.remove(&key);
    }
}

pub fn get_error(app_state: &AppState, key: &str) -> Option<UIError> {
    app_state.error_states.get(key).cloned()
}

pub fn set_error(app_state: &mut AppState, key: String, error: Option<UIError>) {
    if let Some(error) = error {
        app_state.error_states.insert(key, error);
    } else {
        app_state.error_states.remove(&key);
    }
}

pub fn use_expression_input() -> (String, Option<UIError>, Option<ExprId>, Signal<InputState>) {
    let input_state = use_input_state();
    let text = input_state.read().expr_text.clone();
    let error = input_state.read().expr_error.clone();
    let result = input_state.read().expr_result;
    (text, error, result, input_state)
}

pub fn use_ruleset_input() -> (
    String,
    Option<UIError>,
    Option<RulesetId>,
    Signal<InputState>,
) {
    let input_state = use_input_state();
    let text = input_state.read().ruleset_text.clone();
    let error = input_state.read().ruleset_error.clone();
    let result = input_state.read().ruleset_result;
    (text, error, result, input_state)
}

pub fn use_parsing_state() -> (bool, Signal<InputState>) {
    let input_state = use_input_state();
    let blocked = input_state.read().parsing_blocked;
    (blocked, input_state)
}

pub fn use_highlighted_subexpr() -> (Option<ExprId>, Signal<ExpressionState>) {
    let expression_state = use_expression_state();
    let highlighted = expression_state.read().highlighted_subexpr;
    (highlighted, expression_state)
}

pub fn use_rule_hover() -> (Option<usize>, Signal<RulesState>) {
    let rules_state = use_rules_state();
    let hovered = rules_state.read().hovered_rule_index;
    (hovered, rules_state)
}

pub fn use_rule_matches() -> Vec<Match> {
    let rules_state = use_rules_state();
    rules_state.read().current_matches.clone()
}

pub fn use_rule_history() -> Vec<(ExprId, RuleId, ExprId)> {
    let rules_state = use_rules_state();
    rules_state.read().applied_rules_history.clone()
}
