use super::super::{
    display_components::{ExpressionCard, ParsedRulesetsList, PrebuiltRulesets},
    file_utils,
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


    rsx! {
        div { class: styles::SPACE_Y_6,
            div {
                h1 { class: styles::TITLE_PAGE, "Expression Explorer Setup" }
                p { class: styles::TEXT_BODY,
                    if parsing_blocked {
                        "Expression ready for exploration"
                    } else if input_state.read().selected_ruleset.is_none() {
                        "Step 1: Parse and select a ruleset first"
                    } else {
                        "Step 2: Parse your mathematical expression"
                    }
                }
            }
            
            // Workflow progress indicator
            div { class: "flex items-center justify-center gap-2 mb-6",
                div { class: if !input_state.read().parsed_rulesets.is_empty() { 
                    "flex items-center gap-2 text-green-600 font-medium" 
                } else { 
                    "flex items-center gap-2 text-gray-400" 
                },
                    div { class: if !input_state.read().parsed_rulesets.is_empty() { 
                        "w-8 h-8 rounded-full bg-green-600 text-white flex items-center justify-center text-sm" 
                    } else { 
                        "w-8 h-8 rounded-full bg-gray-300 text-white flex items-center justify-center text-sm" 
                    },
                        "1"
                    }
                    span { class: "text-sm", "Parse Ruleset" }
                }
                
                div { class: "w-8 border-t-2 border-gray-300" }
                
                div { class: if input_state.read().selected_ruleset.is_some() { 
                    "flex items-center gap-2 text-green-600 font-medium" 
                } else { 
                    "flex items-center gap-2 text-gray-400" 
                },
                    div { class: if input_state.read().selected_ruleset.is_some() { 
                        "w-8 h-8 rounded-full bg-green-600 text-white flex items-center justify-center text-sm" 
                    } else { 
                        "w-8 h-8 rounded-full bg-gray-300 text-white flex items-center justify-center text-sm" 
                    },
                        "2"
                    }
                    span { class: "text-sm", "Select Ruleset" }
                }
                
                div { class: "w-8 border-t-2 border-gray-300" }
                
                div { class: if expr_result.is_some() { 
                    "flex items-center gap-2 text-green-600 font-medium" 
                } else { 
                    "flex items-center gap-2 text-gray-400" 
                },
                    div { class: if expr_result.is_some() { 
                        "w-8 h-8 rounded-full bg-green-600 text-white flex items-center justify-center text-sm" 
                    } else { 
                        "w-8 h-8 rounded-full bg-gray-300 text-white flex items-center justify-center text-sm" 
                    },
                        "3"
                    }
                    span { class: "text-sm", "Parse Expression" }
                }
                
                div { class: "w-8 border-t-2 border-gray-300" }
                
                div { class: if expr_result.is_some() && input_state.read().selected_ruleset.is_some() { 
                    "flex items-center gap-2 text-green-600 font-medium" 
                } else { 
                    "flex items-center gap-2 text-gray-400" 
                },
                    div { class: if expr_result.is_some() && input_state.read().selected_ruleset.is_some() { 
                        "w-8 h-8 rounded-full bg-green-600 text-white flex items-center justify-center text-sm" 
                    } else { 
                        "w-8 h-8 rounded-full bg-gray-300 text-white flex items-center justify-center text-sm" 
                    },
                        "4"
                    }
                    span { class: "text-sm", "Explore" }
                }
            }

            if let Some(expr_id) = expr_result.as_ref() {
                div { class: "mb-6",
                    ExpressionCard {
                        pool: pool,
                        expr_id: *expr_id,
                        highlighted_subexpr: None,
                    }

                    div { class: "flex gap-4 mt-4",
                        button {
                            class: styles::BTN_SECONDARY,
                            onclick: move |_| {
                                let mut state = input_state.write();
                                state.parsing_blocked = false;
                                state.expr_result = None;
                                state.expr_error = None;
                                state.selected_ruleset = None;
                            },
                            "← Reset to Parse More"
                        }

                        if input_state.read().selected_ruleset.is_some() {
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
                    // Step 1: Ruleset Input (Always visible)
                    div { class: styles::PANEL,
                        h2 { class: styles::TITLE_SECTION, "Step 1: Ruleset Input" }

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
                                div { class: "flex items-center justify-between mb-2",
                                    label { class: styles::LABEL, "Transformation Rules" }
                                    div { class: "flex gap-2",
                                        // Upload button
                                        label {
                                            class: format!("{} text-sm", styles::BTN_SECONDARY),
                                            "for": "ruleset-upload",
                                            "📁 Upload"
                                        }
                                        input {
                                            id: "ruleset-upload",
                                            r#type: "file",
                                            accept: ".txt,.rules",
                                            class: "hidden",
                                            onchange: move |evt| {
                                                spawn(async move {
                                                    if let Some((_filename, contents)) = file_utils::read_file_from_event(&evt).await {
                                                        let mut state = input_state.write();
                                                        state.ruleset_text = contents.clone();
                                                        state.ruleset_error = None;
                                                        state.ruleset_result = None;
                                                        
                                                        // Auto-parse the uploaded ruleset
                                                        let mut p = pool.write();
                                                        match parser::parse_ruleset(&contents, &mut p) {
                                                            Ok(ruleset_id) => {
                                                                state.ruleset_error = None;
                                                                state.ruleset_result = Some(ruleset_id);
                                                                if !state.parsed_rulesets.contains(&ruleset_id) {
                                                                    state.parsed_rulesets.push(ruleset_id);
                                                                }
                                                                if state.selected_ruleset.is_none() {
                                                                    state.selected_ruleset = Some(ruleset_id);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                state.ruleset_error = Some(UIError::ParseError {
                                                                    message: format!("Ruleset: {}", e),
                                                                    position: None
                                                                });
                                                                state.ruleset_result = None;
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        
                                        // Download button
                                        button {
                                            class: format!("{} text-sm", styles::BTN_SECONDARY),
                                            disabled: ruleset_text.is_empty(),
                                            onclick: move |_| {
                                                let text = input_state.read().ruleset_text.clone();
                                                file_utils::download_text_file("ruleset.txt", &text);
                                            },
                                            "💾 Download"
                                        }
                                    }
                                }
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
                                            // Add to parsed rulesets list if not already present
                                            if !state.parsed_rulesets.contains(&ruleset_id) {
                                                state.parsed_rulesets.push(ruleset_id);
                                            }
                                            // Auto-select if it's the first parsed ruleset
                                            if state.selected_ruleset.is_none() {
                                                state.selected_ruleset = Some(ruleset_id);
                                            }
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
                            
                            ParsedRulesetsList {
                                pool: pool,
                                parsed_rulesets: input_state.read().parsed_rulesets.clone(),
                                selected_ruleset: input_state.read().selected_ruleset,
                                on_ruleset_selected: move |ruleset_id| {
                                    input_state.write().selected_ruleset = Some(ruleset_id);
                                }
                            }
                        }
                    }

                    // Step 2: Expression Input (Only enabled after ruleset selection)
                    div { class: if input_state.read().selected_ruleset.is_some() {
                        styles::PANEL.to_string()
                    } else {
                        format!("{} opacity-50 pointer-events-none", styles::PANEL)
                    },
                        h2 { class: styles::TITLE_SECTION, "Step 2: Expression Input" }

                        if input_state.read().selected_ruleset.is_none() {
                            div { class: "text-sm text-orange-600 mb-4 p-3 bg-orange-50 rounded-lg border border-orange-200",
                                "⚠️ Please parse and select a ruleset before entering an expression"
                            }
                        }
                        
                        div { class: styles::SPACE_Y_4,
                            div {
                                div { class: "flex items-center justify-between mb-2",
                                    label { class: styles::LABEL, "Mathematical Expression" }
                                    div { class: "flex gap-2",
                                        // Upload button
                                        label {
                                            class: format!("{} text-sm", styles::BTN_SECONDARY),
                                            "for": "expr-upload",
                                            "📁 Upload"
                                        }
                                        input {
                                            id: "expr-upload",
                                            r#type: "file",
                                            accept: ".txt,.expr",
                                            class: "hidden",
                                            onchange: move |evt| {
                                                spawn(async move {
                                                    if let Some((_filename, contents)) = file_utils::read_file_from_event(&evt).await {
                                                        let mut state = input_state.write();
                                                        state.expr_text = contents.clone();
                                                        state.expr_error = None;
                                                        state.expr_result = None;
                                                        
                                                        // Auto-parse the uploaded expression if we have a selected ruleset
                                                        if state.selected_ruleset.is_some() {
                                                            let mut p = pool.write();
                                                            match parser::expr::parse_expression(&contents, &mut p) {
                                                                Ok(expr_id) => {
                                                                    state.expr_error = None;
                                                                    state.expr_result = Some(expr_id);
                                                                    state.parsing_blocked = true;
                                                                    // Navigate to explorer page after successful parse
                                                                    on_navigate.call(Page::Explorer);
                                                                }
                                                                Err(e) => {
                                                                    state.expr_error = Some(UIError::ParseError {
                                                                        message: e.to_string(),
                                                                        position: None
                                                                    });
                                                                    state.expr_result = None;
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        
                                        // Download button
                                        button {
                                            class: format!("{} text-sm", styles::BTN_SECONDARY),
                                            disabled: expr_text.is_empty(),
                                            onclick: move |_| {
                                                let text = input_state.read().expr_text.clone();
                                                file_utils::download_text_file("expression.txt", &text);
                                            },
                                            "💾 Download"
                                        }
                                    }
                                }
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
                                class: if input_state.read().selected_ruleset.is_some() {
                                    styles::BTN_PRIMARY.to_string()
                                } else {
                                    format!("{} opacity-50 cursor-not-allowed", styles::BTN_PRIMARY)
                                },
                                disabled: input_state.read().selected_ruleset.is_none(),
                                onclick: move |_| {
                                    if input_state.read().selected_ruleset.is_none() {
                                        return;
                                    }
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
                }
            }

            div { class: styles::PANEL,
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: styles::TITLE_SUBSECTION, "Pool Status" }
                    div { class: "flex gap-2",
                        // Upload pool button
                        label {
                            class: format!("{} text-sm", styles::BTN_SECONDARY),
                            "for": "pool-upload",
                            "📁 Import Pool"
                        }
                        input {
                            id: "pool-upload",
                            r#type: "file",
                            accept: ".json",
                            class: "hidden",
                            onchange: move |evt| {
                                spawn(async move {
                                    if let Some((_filename, contents)) = file_utils::read_file_from_event(&evt).await {
                                        match serde_json::from_str::<crate::Pool>(&contents) {
                                            Ok(imported_pool) => {
                                                pool.set(imported_pool);
                                                log::info!("Pool imported successfully");
                                                // Navigate to explorer page after successful import
                                                on_navigate.call(Page::Explorer);
                                            }
                                            Err(e) => {
                                                log::error!("Failed to import pool: {}", e);
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        
                        // Export pool button
                        button {
                            class: format!("{} text-sm", styles::BTN_SECONDARY),
                            onclick: move |_| {
                                let pool_ref = pool.read();
                                
                                // Serialize the entire pool to JSON
                                match serde_json::to_string_pretty(&*pool_ref) {
                                    Ok(json_content) => {
                                        file_utils::download_text_file("pool_export.json", &json_content);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to serialize pool: {}", e);
                                    }
                                }
                            },
                            "💾 Export Pool"
                        }
                    }
                }
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