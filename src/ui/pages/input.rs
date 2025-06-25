use super::super::{
    display_components::{ExpressionCard, PrebuiltRulesets, ViewMode},
    navigation::Page,
    primitives::{ErrorDisplay, UIError},
    state::{InputStateProvider, use_expression_input, use_parsing_state, use_ruleset_input},
    styles,
};
use crate::{Pool, parser};
use dioxus::prelude::*;

#[component]
pub fn InputPage(pool: Signal<Pool>, on_navigate: EventHandler<Page>) -> Element {
    rsx! {
        InputStateProvider {
            InputPageContent { pool: pool, on_navigate: on_navigate }
        }
    }
}

#[component]
fn InputPageContent(pool: Signal<Pool>, on_navigate: EventHandler<Page>) -> Element {
    let (expr_text, expr_error, expr_result, mut input_state) = use_expression_input();
    let (ruleset_text, ruleset_error, ruleset_result, _ruleset_state) = use_ruleset_input();
    let (parsing_blocked, _parsing_state) = use_parsing_state();

    let mut view_mode = use_signal(|| ViewMode::Text);

    rsx! {
        div { class: styles::SPACE_Y_6,
            div {
                h1 { class: styles::TITLE_PAGE, "Expression & Ruleset Input" }
                p { class: styles::TEXT_BODY,
                    if parsing_blocked {
                        "Parsing blocked - expression ready for exploration"
                    } else {
                        "Parse mathematical expressions and transformation rules"
                    }
                }
            }

            if let Some(expr_id) = expr_result.as_ref() {
                div { class: "mb-6",
                    ExpressionCard {
                        pool: pool,
                        expr_id: *expr_id,
                        view_mode: *view_mode.read(),
                        highlighted_subexpr: None,
                        on_view_mode_change: move |new_mode| {
                            view_mode.set(new_mode);
                        },
                    }

                    div { class: "flex gap-4 mt-4",
                        button {
                            class: styles::BTN_SECONDARY,
                            onclick: move |_| {
                                let mut state = input_state.write();
                                state.parsing_blocked = false;
                                state.expr_result = None;
                                state.expr_error = None;
                            },
                            "← Reset to Parse More"
                        }

                        if ruleset_result.is_some() {
                            button {
                                class: styles::BTN_PRIMARY,
                                onclick: move |_| {
                                    on_navigate.call(Page::Explorer);
                                },
                                "🔍 Explore with Rules →"
                            }
                        }
                    }
                }
            }

            if !parsing_blocked {
                div { class: styles::GRID_2,
                div { class: styles::PANEL,
                    h2 { class: styles::TITLE_SECTION, "Expression Input" }

                    div { class: styles::SPACE_Y_4,
                        div {
                            label { class: styles::LABEL, "Mathematical Expression" }
                            textarea {
                                class: styles::TEXTAREA,
                                placeholder: "Enter mathematical expression (e.g., (x + y) * 2, sin(x + 1), Point{{x, y}})",
                                value: expr_text.clone(),
                                oninput: move |evt| {
                                    let mut state = input_state.write();
                                    state.expr_text = evt.value();
                                    state.expr_error = None;
                                    state.expr_result = None;
                                }
                            }
                        }

                        button {
                            class: styles::BTN_PRIMARY,
                            onclick: move |_| {
                                let mut p = pool.write();
                                let current_text = input_state.read().expr_text.clone();
                                match parser::expr::parse_expression(&current_text, &mut p) {
                                    Ok(expr_id) => {
                                        let mut state = input_state.write();
                                        state.expr_error = None;
                                        state.expr_result = Some(expr_id);
                                        state.parsing_blocked = true;
                                    }
                                    Err(e) => {
                                        let ui_error = UIError::ParseError {
                                            message: e.to_string(),
                                            position: None
                                        };
                                        let mut state = input_state.write();
                                        state.expr_error = Some(ui_error);
                                        state.expr_result = None;
                                    }
                                }
                            },
                            "Parse Expression"
                        }

                        if let Some(error) = expr_error.as_ref() {
                            ErrorDisplay {
                                error: error.clone(),
                                show_details: false,
                                show_retry: true,
                                on_retry: Some(EventHandler::new(move |_| {
                                    input_state.write().expr_error = None;
                                })),
                                on_dismiss: Some(EventHandler::new(move |_| {
                                    input_state.write().expr_error = None;
                                })),
                            }
                        }
                    }
                }

                div { class: styles::PANEL,
                    h2 { class: styles::TITLE_SECTION, "Ruleset Input" }

                    div { class: styles::SPACE_Y_4,
                        PrebuiltRulesets {
                            on_ruleset_selected: move |ruleset| {
                                let mut state = input_state.write();
                                state.ruleset_text = ruleset;
                                state.ruleset_error = None;
                                state.ruleset_result = None;
                            }
                        }

                        div {
                            label { class: styles::LABEL, "Transformation Rules" }
                            textarea {
                                class: styles::TEXTAREA_LG,
                                placeholder: "Enter ruleset (e.g., algebra {{ rule_name: ?pattern => action }})",
                                value: ruleset_text.clone(),
                                oninput: move |evt| {
                                    let mut state = input_state.write();
                                    state.ruleset_text = evt.value();
                                    state.ruleset_error = None;
                                    state.ruleset_result = None;
                                }
                            }
                        }

                        button {
                            class: styles::BTN_SUCCESS,
                            onclick: move |_| {
                                let mut p = pool.write();
                                let current_text = input_state.read().ruleset_text.clone();
                                match parser::parse_ruleset(&current_text, &mut p) {
                                    Ok(ruleset_id) => {
                                        let mut state = input_state.write();
                                        state.ruleset_error = None;
                                        state.ruleset_result = Some(ruleset_id);
                                    }
                                    Err(e) => {
                                        let ui_error = UIError::ParseError {
                                            message: format!("Ruleset: {}", e),
                                            position: None
                                        };
                                        let mut state = input_state.write();
                                        state.ruleset_error = Some(ui_error);
                                        state.ruleset_result = None;
                                    }
                                }
                            },
                            "Parse Ruleset"
                        }

                        if let Some(error) = ruleset_error.as_ref() {
                            ErrorDisplay {
                                error: error.clone(),
                                show_details: false,
                                show_retry: true,
                                on_retry: Some(EventHandler::new(move |_| {
                                    input_state.write().ruleset_error = None;
                                })),
                                on_dismiss: Some(EventHandler::new(move |_| {
                                    input_state.write().ruleset_error = None;
                                })),
                            }
                        }

                        if let Some(ruleset_id) = ruleset_result.as_ref() {
                            div { class: styles::SUCCESS_BOX,
                                "✓ Ruleset parsed! {pool.read().get_ruleset_rule_count(*ruleset_id)} rules loaded"
                            }
                        }
                    }
                }
            }
            }

            div { class: styles::PANEL,
                h3 { class: styles::TITLE_SUBSECTION, "Pool Status" }
                div { class: styles::GRID_3,
                    div { class: styles::CARD,
                        div { class: styles::TEXT_SMALL, "Expressions" }
                        div { class: styles::TITLE_SUBSECTION, "{pool.read().exprs.len()}" }
                    }
                    div { class: styles::CARD,
                        div { class: styles::TEXT_SMALL, "Rules" }
                        div { class: styles::TITLE_SUBSECTION, "{pool.read().rules.len()}" }
                    }
                    div { class: styles::CARD,
                        div { class: styles::TEXT_SMALL, "Rulesets" }
                        div { class: styles::TITLE_SUBSECTION, "{pool.read().rulesets.len()}" }
                    }
                }
            }
        }
    }
}
