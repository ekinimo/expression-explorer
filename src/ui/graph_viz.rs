use super::{
    primitives::{ExpressionTreeGraph, TransformationGraph},
    styles,
};
use crate::{DisplayNode, ExprId, Pool};
use dioxus::prelude::*;

#[component]
pub fn SimpleExpressionGraph(pool: Signal<Pool>, current_expr: Option<ExprId>) -> Element {
    rsx! {
        div { class: "h-full w-full flex flex-col",
            TransformationGraph {
                pool: pool,
                current_expr: current_expr,
            }
        }
    }
}

#[component]
pub fn MiniExpressionGraph(pool: Signal<Pool>, expr_id: ExprId) -> Element {
    rsx! {
        ExpressionTreeGraph {
            pool: pool,
            expr_id: expr_id,
            mini: true,
        }
    }
}

#[component]
pub fn ExpressionList(pool: Signal<Pool>, on_expression_selected: EventHandler<ExprId>) -> Element {
    let pool_ref = pool.read();
    let expressions: Vec<_> = (0..pool_ref.exprs.len()).map(ExprId).collect();

    rsx! {
        div { class: styles::PANEL,
            h3 { class: styles::TITLE_SECTION, "Available Expressions" }

            if expressions.is_empty() {
                div { class: styles::TEXT_MUTED, "No expressions available" }
            } else {
                div { class: styles::SPACE_Y_2,
                    for expr_id in expressions {
                        ExpressionItem {
                            pool: pool,
                            expr_id: expr_id,
                            on_click: on_expression_selected,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ExpressionItem(pool: Signal<Pool>, expr_id: ExprId, on_click: EventHandler<ExprId>) -> Element {
    let pool_ref = pool.read();
    let expr_text = pool_ref.display_with_children(expr_id);

    rsx! {
        div {
            class: styles::CARD,
            style: "cursor: pointer; transition: all 0.2s;",
            onclick: move |_| on_click.call(expr_id),

            div { class: styles::FLEX_BETWEEN,
                div {
                    div { class: styles::TEXT_MONO, "{expr_text}" }
                    div { class: styles::TEXT_TINY, "ID: {expr_id:?}" }
                }
                div { class: styles::BTN_SM,
                    "View"
                }
            }
        }
    }
}
