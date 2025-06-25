use dioxus::prelude::*;

#[component]
pub fn HowToPage() -> Element {
    rsx! {
        div { class: "max-w-4xl mx-auto p-8 space-y-8",
            div { class: "text-center mb-8",
                h1 { class: "text-3xl font-bold text-gray-800 mb-2", "How to Use Expression Explorer" }
                p { class: "text-gray-600", "A guide to exploring and transforming mathematical expressions" }
            }

            Section {
                title: "Getting Started",
                content: rsx! {
                    div { class: "space-y-4",
                        p { "Expression Explorer helps you visualize and transform mathematical expressions using rewrite rules." }

                        div { class: "bg-blue-50 border-l-4 border-blue-400 p-4",
                            p { class: "font-semibold", "Quick Start:" }
                            ol { class: "list-decimal list-inside mt-2 space-y-1",
                                li { "Go to the Input page to enter expressions and rulesets" }
                                li { "Navigate to Explorer to visualize and transform expressions" }
                            }
                        }
                    }
                }
            }

            Section {
                title: "Input Page - Parsing Expressions and Rules",
                content: rsx! {
                    div { class: "space-y-4",
                        h3 { class: "font-semibold text-lg", "Entering Expressions" }
                        p { "Enter mathematical expressions using standard notation:" }

                        CodeExample {
                            code: "(x + y) * 2\na * (b + c)\nsqrt(x^2 + y^2)"
                        }

                        h3 { class: "font-semibold text-lg mt-6", "Defining Rulesets" }
                        p { "Create transformation rules using pattern matching:" }

                        CodeExample {
                            code: "algebra {{\n  distribute: ?a * (?b + ?c) => a * b + a * c\n  commute_add: ?x + ?y => y + x\n  identity: ?x + 0 => x\n}}"
                        }

                        div { class: "bg-yellow-50 border-l-4 border-yellow-400 p-4 mt-4",
                            p { class: "font-semibold", "Pattern Types:" }
                            ul { class: "list-disc list-inside mt-2 space-y-1",
                                li { "x, y, a, b - Match exact variables (literal variable names)" }
                                li { "?x, ?y, ?a, ?b - Wildcards (match any expression)" }
                                li { "#x, #y, #a, #b - AnyNumber (match any numeric value)" }
                                li { "Patterns on the left side match expressions" }
                                li { "Actions on the right side define transformations" }
                            }
                        }

                        h3 { class: "font-semibold text-lg mt-6", "Advanced Actions" }
                        p { "Actions can include compute operations for mathematical transformations:" }

                        CodeExample {
                            code: "math {{\n  add_numbers: #a + #b => [a + b]\n  multiply_by_zero: ?x * 0 => 0\n  simplify_double: 2 * #x => [2 * x]\n}}"
                        }

                        div { class: "bg-blue-50 border-l-4 border-blue-400 p-4 mt-4",
                            p { class: "font-semibold", "Action Types:" }
                            ul { class: "list-disc list-inside mt-2 space-y-1",
                                li { "Literal values - Direct replacement (e.g., 0, x + y)" }
                                li { "Variable substitution - Use captured patterns (x, a, b without prefixes)" }
                                li { "[expr] - Compute operations for mathematical evaluation" }
                            }
                        }
                    }
                }
            }

              Section {
                title: "Example Patterns",
                content: rsx! {
                    div { class: "space-y-6",
                        ExampleCard {
                            title: "Algebraic Simplification",
                            expression: "2 * x + 3 * x",
                            rule: "combine_like: #a * ?x + #b * ?x => [a + b] * x",
                            result: "(2 + 3) * x"
                        }

                        ExampleCard {
                            title: "Distribution",
                            expression: "a * (b + c)",
                            rule: "distribute: ?a * (?b + ?c) => a * b + a * c",
                            result: "a * b + a * c"
                        }

                        ExampleCard {
                            title: "Identity Elimination",
                            expression: "x + 0",
                            rule: "identity: ?x + 0 => x",
                            result: "x"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Section(title: &'static str, content: Element) -> Element {
    rsx! {
        div { class: "bg-white rounded-lg shadow-sm border border-gray-200 p-6",
            h2 { class: "text-2xl font-bold text-gray-800 mb-4", "{title}" }
            {content}
        }
    }
}

#[component]
fn CodeExample(code: &'static str) -> Element {
    rsx! {
        pre { class: "bg-gray-100 rounded p-3 overflow-x-auto",
            code { class: "text-sm font-mono", "{code}" }
        }
    }
}

#[component]
fn FeatureCard(title: &'static str, description: &'static str) -> Element {
    rsx! {
        div { class: "bg-gray-50 rounded p-4",
            h4 { class: "font-semibold text-gray-800", "{title}" }
            p { class: "text-gray-600 text-sm mt-1", "{description}" }
        }
    }
}

#[component]
fn VisualizationCard(title: &'static str, description: &'static str) -> Element {
    rsx! {
        div { class: "bg-blue-50 rounded p-4 border border-blue-200",
            h4 { class: "font-semibold text-blue-800", "{title}" }
            p { class: "text-blue-700 text-sm mt-1", "{description}" }
        }
    }
}

#[component]
fn TipCard(icon: &'static str, tip: &'static str) -> Element {
    rsx! {
        div { class: "flex items-start gap-3 bg-green-50 rounded p-3",
            span { class: "text-2xl", "{icon}" }
            p { class: "text-gray-700 text-sm", "{tip}" }
        }
    }
}

#[component]
fn ExampleCard(
    title: &'static str,
    expression: &'static str,
    rule: &'static str,
    result: &'static str,
) -> Element {
    rsx! {
        div { class: "bg-purple-50 rounded p-4 space-y-2",
            h4 { class: "font-semibold text-purple-800", "{title}" }
            div { class: "space-y-1 text-sm",
                div {
                    span { class: "font-semibold text-gray-700", "Expression: " }
                    code { class: "bg-white px-2 py-1 rounded", "{expression}" }
                }
                div {
                    span { class: "font-semibold text-gray-700", "Rule: " }
                    code { class: "bg-white px-2 py-1 rounded text-xs", "{rule}" }
                }
                div {
                    span { class: "font-semibold text-gray-700", "Result: " }
                    code { class: "bg-white px-2 py-1 rounded", "{result}" }
                }
            }
        }
    }
}
