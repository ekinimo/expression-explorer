use super::styles;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum Page {
    Input,
    Explorer,
    Debug,
    HowTo,
}

impl Page {
    pub fn title(&self) -> &'static str {
        match self {
            Page::Input => "Input",
            Page::Explorer => "Explorer",
            Page::Debug => "Debug",
            Page::HowTo => "How To",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Page::Input => "Parse expressions and rulesets",
            Page::Explorer => "Find and apply transformation rules with graph visualization",
            Page::Debug => "Inspect pool state and debug parsing",
            Page::HowTo => "Learn how to use Expression Explorer",
        }
    }
}

#[component]
pub fn NavigationBar(current_page: Signal<Page>, on_page_change: EventHandler<Page>) -> Element {
    let pages = [Page::Input, Page::Explorer, Page::Debug, Page::HowTo];

    rsx! {
        nav { class: styles::NAV_BAR,
            div { class: styles::NAV_CONTAINER,
                div { class: styles::FLEX_BETWEEN,
                    div { class: styles::NAV_TITLE,
                        "Expression Explorer"
                    }

                    div { class: styles::NAV_LIST,
                        for page in pages {
                            NavigationItem {
                                page: page,
                                current_page: current_page.read().clone(),
                                on_click: move |p| on_page_change.call(p),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavigationItem(page: Page, current_page: Page, on_click: EventHandler<Page>) -> Element {
    let is_active = page == current_page;
    let class = if is_active {
        styles::NAV_ITEM_ACTIVE
    } else {
        styles::NAV_ITEM_INACTIVE
    };

    let description = page.description();
    let title = page.title();

    rsx! {
        div {
            class: class,
            onclick: move |_| on_click.call(page.clone()),
            title: description,

            "{title}"
        }
    }
}
