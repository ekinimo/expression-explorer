use crate::{Pool, ExprId, DisplayNode, Children};
use dioxus::prelude::*;
use super::super::{styles, graph_viz::ExpressionList, primitives::TransformationGraph};

#[component]
pub fn GraphPage(pool: Signal<Pool>) -> Element {
    let mut selected_expr = use_signal(|| None::<ExprId>);

    rsx! {
        div { class: styles::SPACE_Y_6,
            div {
                h1 { class: styles::TITLE_PAGE, "Expression Graph" }
                p { class: styles::TEXT_BODY, "Visualize expression structures and navigate between expressions" }
            }

            div { class: styles::GRID_2,
                ExpressionList {
                    pool: pool,
                    on_expression_selected: move |expr_id| {
                        selected_expr.set(Some(expr_id));
                    },
                }

                TransformationGraph {
                    pool: pool,
                    current_expr: selected_expr.read().clone(),
                }
            }

            if let Some(expr_id) = selected_expr.read().as_ref() {
                div { class: styles::PANEL,
                    h3 { class: styles::TITLE_SUBSECTION, "Selected Expression Details" }
                    div { class: styles::SPACE_Y_2,
                        div { class: styles::FLEX_ROW,
                            span { class: styles::LABEL_INLINE, "Expression ID:" }
                            span { class: styles::TEXT_MONO, "{expr_id:?}" }
                        }
                        div { class: styles::FLEX_ROW,
                            span { class: styles::LABEL_INLINE, "Display:" }
                            span { class: styles::TEXT_MONO, "{pool.read().display_with_children(*expr_id)}" }
                        }
                        div { class: styles::FLEX_ROW,
                            span { class: styles::LABEL_INLINE, "Children:" }
                            span { class: styles::TEXT_SMALL, "{pool.read().children(*expr_id).count()}" }
                        }
                    }
                }
            }
        }
    }
}