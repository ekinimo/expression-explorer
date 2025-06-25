use super::super::{display_components::*, styles};
use crate::{ActionId, ExprId, PatternId, Pool, RuleId};
use dioxus::prelude::*;

#[component]
pub fn DebugPage(pool: Signal<Pool>) -> Element {
    rsx! {
        div { class: styles::SPACE_Y_6,
            div {
                h1 { class: styles::TITLE_PAGE, "Debug Console" }
                p { class: styles::TEXT_BODY, "Inspect pool state, test parsing, and debug the expression system" }
            }

            div { class: styles::PANEL,
                h2 { class: styles::TITLE_SECTION, "Pool State" }

                div { class: styles::SPACE_Y_4,
                    div { class: styles::GRID_3,
                        div { class: styles::CARD,
                            div { class: styles::LABEL_INLINE, "Expressions:" }
                            div { class: styles::TEXT_MONO, "{pool.read().exprs.len()}" }
                        }
                        div { class: styles::CARD,
                            div { class: styles::LABEL_INLINE, "Patterns:" }
                            div { class: styles::TEXT_MONO, "{pool.read().patterns.len()}" }
                        }
                        div { class: styles::CARD,
                            div { class: styles::LABEL_INLINE, "Actions:" }
                            div { class: styles::TEXT_MONO, "{pool.read().actions.len()}" }
                        }
                        div { class: styles::CARD,
                            div { class: styles::LABEL_INLINE, "Rules:" }
                            div { class: styles::TEXT_MONO, "{pool.read().rules.len()}" }
                        }
                        div { class: styles::CARD,
                            div { class: styles::LABEL_INLINE, "Rulesets:" }
                            div { class: styles::TEXT_MONO, "{pool.read().rulesets.len()}" }
                        }
                        div { class: styles::CARD,
                            div { class: styles::LABEL_INLINE, "Names:" }
                            div { class: styles::TEXT_MONO, "{pool.read().names.len()}" }
                        }
                    }

                    div {
                        h3 { class: styles::TITLE_SUBSECTION, "Pool Contents" }

                        if !pool.read().exprs.is_empty() {
                            div { class: styles::SPACE_Y_4,
                                h4 { class: "text-lg font-medium text-gray-700", "Expressions (First 5)" }
                                div { class: styles::SPACE_Y_2,
                                    for i in 0..pool.read().exprs.len().min(5) {
                                        ExpressionDisplay {
                                            pool: pool,
                                            expr_id: ExprId(i),
                                            highlight: false,
                                        }
                                    }
                                    if pool.read().exprs.len() > 5 {
                                        div { class: styles::TEXT_MUTED,
                                            "... and {pool.read().exprs.len() - 5} more expressions"
                                        }
                                    }
                                }
                            }
                        }

                        if !pool.read().patterns.is_empty() {
                            div { class: styles::SPACE_Y_4,
                                h4 { class: "text-lg font-medium text-gray-700", "Patterns (First 3)" }
                                div { class: styles::SPACE_Y_2,
                                    for i in 0..pool.read().patterns.len().min(3) {
                                        PatternDisplay {
                                            pool: pool,
                                            pattern_id: PatternId(i),
                                            highlight: false,
                                        }
                                    }
                                    if pool.read().patterns.len() > 3 {
                                        div { class: styles::TEXT_MUTED,
                                            "... and {pool.read().patterns.len() - 3} more patterns"
                                        }
                                    }
                                }
                            }
                        }

                        if !pool.read().actions.is_empty() {
                            div { class: styles::SPACE_Y_4,
                                h4 { class: "text-lg font-medium text-gray-700", "Actions (First 3)" }
                                div { class: styles::SPACE_Y_2,
                                    for i in 0..pool.read().actions.len().min(3) {
                                        ActionDisplay {
                                            pool: pool,
                                            action_id: ActionId(i),
                                            highlight: false,
                                        }
                                    }
                                    if pool.read().actions.len() > 3 {
                                        div { class: styles::TEXT_MUTED,
                                            "... and {pool.read().actions.len() - 3} more actions"
                                        }
                                    }
                                }
                            }
                        }

                        if !pool.read().rules.is_empty() {
                            div { class: styles::SPACE_Y_4,
                                h4 { class: "text-lg font-medium text-gray-700", "Rules (First 3)" }
                                div { class: styles::SPACE_Y_2,
                                    for i in 0..pool.read().rules.len().min(3) {
                                        RuleDisplay {
                                            pool: pool,
                                            rule_id: RuleId(i),
                                            highlight: false,
                                        }
                                    }
                                    if pool.read().rules.len() > 3 {
                                        div { class: styles::TEXT_MUTED,
                                            "... and {pool.read().rules.len() - 3} more rules"
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
