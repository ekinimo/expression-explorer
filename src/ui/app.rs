use super::{
    navigation::{NavigationBar, Page},
    pages::{DebugPage, ExplorerPage, HowToPage, InputPage},
    primitives::ErrorBoundary,
    state::AppStateProvider,
    styles,
};
use crate::Pool;
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! {
        AppStateProvider {
            AppContent {}
        }
    }
}

#[component]
fn AppContent() -> Element {
    let mut current_page = use_signal(|| Page::Input);
    let pool = use_signal(Pool::new);

    rsx! {
        div { class: styles::APP_CONTAINER,
            NavigationBar {
                current_page: current_page,
                on_page_change: move |page| {
                    current_page.set(page);
                },
            }

            div { class: styles::CONTENT_CONTAINER,
                ErrorBoundary {
                    fallback_message: "A page error occurred. Please try navigating to a different page.".to_string(),
                    show_details: true,
                    {
                        match current_page.read().clone() {
                            Page::Input => rsx! {
                                ErrorBoundary {
                                    fallback_message: "Input page encountered an error.".to_string(),
                                    InputPage {
                                        pool: pool,
                                        on_navigate: move |page| current_page.set(page),
                                    }
                                }
                            },
                            Page::Explorer => rsx! {
                                ErrorBoundary {
                                    fallback_message: "Explorer page encountered an error.".to_string(),
                                    ExplorerPage { pool: pool }
                                }
                            },
                            Page::Debug => rsx! {
                                ErrorBoundary {
                                    fallback_message: "Debug page encountered an error.".to_string(),
                                    DebugPage { pool: pool }
                                }
                            },
                            Page::HowTo => rsx! {
                                ErrorBoundary {
                                    fallback_message: "How To page encountered an error.".to_string(),
                                    HowToPage {}
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
