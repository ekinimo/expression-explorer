use super::styles;
use dioxus::prelude::*;

#[component]
pub fn LoadingSpinner() -> Element {
    rsx! {
        div { class: styles::FLEX_CENTER,
            div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" }
        }
    }
}

#[component]
pub fn ErrorMessage(message: String) -> Element {
    rsx! {
        div { class: styles::ERROR_BOX,
            "Error: {message}"
        }
    }
}

#[component]
pub fn SuccessMessage(message: String) -> Element {
    rsx! {
        div { class: styles::SUCCESS_BOX,
            "✓ {message}"
        }
    }
}

#[component]
pub fn InfoMessage(message: String) -> Element {
    rsx! {
        div { class: styles::INFO_BOX,
            "ℹ {message}"
        }
    }
}
