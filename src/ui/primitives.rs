use crate::{Children, DisplayNode};
use dioxus::prelude::*;

pub trait Styled {
    fn base_class() -> &'static str;
    fn variant_class(&self) -> &'static str;
    fn state_class(&self) -> &'static str;
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputFieldState {
    Default,
    Focused,
    Valid,
    Invalid,
    Disabled,
}

#[component]
pub fn InputField(
    label: String,
    placeholder: String,
    value: String,
    field_type: String,
    on_change: EventHandler<String>,
    #[props(default = InputFieldState::Default)] state: InputFieldState,
) -> Element {
    let handle_input = move |new_value: String| {
        on_change.call(new_value);
    };

    let input_class = match state {
        InputFieldState::Default => "border-gray-300 focus:border-blue-500",
        InputFieldState::Focused => "border-blue-500 ring-2 ring-blue-200",
        InputFieldState::Valid => "border-green-500 bg-green-50",
        InputFieldState::Invalid => "border-red-500 bg-red-50",
        InputFieldState::Disabled => "border-gray-200 bg-gray-100 cursor-not-allowed",
    };

    rsx! {
        div { class: "space-y-2",
            label { class: "block text-sm font-medium text-gray-700",
                "{label}"
                span { class: "text-xs text-gray-500 ml-2", "({field_type})" }
            }

            input {
                class: "w-full px-3 py-2 border rounded-lg transition-colors {input_class}",
                placeholder: placeholder,
                value: value,
                disabled: matches!(state, InputFieldState::Disabled),
                oninput: move |evt| handle_input(evt.value()),
                onfocus: move |_| {
                },
            }

        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ButtonState {
    Ready,
    Processing,
    Disabled,
    Success,
    Error,
}

#[component]
pub fn ParseButton(
    text: String,
    button_type: String,
    can_parse: bool,
    on_click: EventHandler<()>,
    #[props(default = ButtonState::Ready)] state: ButtonState,
) -> Element {
    let button_class = match state {
        ButtonState::Ready => "bg-blue-500 hover:bg-blue-600 text-white",
        ButtonState::Processing => "bg-yellow-500 text-white cursor-wait",
        ButtonState::Disabled => "bg-gray-300 text-gray-500 cursor-not-allowed",
        ButtonState::Success => "bg-green-500 text-white",
        ButtonState::Error => "bg-red-500 text-white",
    };

    let icon = match state {
        ButtonState::Ready => "▶",
        ButtonState::Processing => "⏳",
        ButtonState::Disabled => "⏸",
        ButtonState::Success => "✓",
        ButtonState::Error => "✗",
    };

    rsx! {
        button {
            class: "px-4 py-2 rounded-lg font-medium transition-colors {button_class}",
            disabled: !can_parse || matches!(state, ButtonState::Disabled | ButtonState::Processing),
            onclick: move |_| on_click.call(()),

            span { class: "mr-2", "{icon}" }
            "Parse {button_type}"

            if matches!(state, ButtonState::Processing) {
                span { class: "ml-2 text-xs", "(disabled until reset)" }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}

impl StatusType {
    fn class(&self) -> &'static str {
        match self {
            StatusType::Info => "bg-blue-50 border-blue-200 text-blue-800",
            StatusType::Success => "bg-green-50 border-green-200 text-green-800",
            StatusType::Warning => "bg-yellow-50 border-yellow-200 text-yellow-800",
            StatusType::Error => "bg-red-50 border-red-200 text-red-800",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            StatusType::Info => "ℹ",
            StatusType::Success => "✓",
            StatusType::Warning => "⚠",
            StatusType::Error => "✗",
        }
    }
}

#[component]
pub fn StatusPanel(
    title: String,
    message: String,
    status_type: StatusType,
    #[props(default = false)] dismissible: bool,
    on_dismiss: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        div { class: "border rounded-lg p-4 {status_type.class()}",
            div { class: "flex items-start justify-between",
                div { class: "flex items-start space-x-3",
                    div { class: "text-lg", "{status_type.icon()}" }
                    div {
                        div { class: "font-semibold text-sm", "{title}" }
                        div { class: "text-sm mt-1", "{message}" }
                    }
                }

                if dismissible {
                    if let Some(dismiss_handler) = on_dismiss {
                        button {
                            class: "text-lg hover:opacity-70",
                            onclick: move |_| dismiss_handler.call(()),
                            "×"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PoolDisplay(pool: Signal<crate::Pool>) -> Element {
    let pool_ref = pool.read();

    let stats = vec![
        (
            "Expressions".to_string(),
            pool_ref.exprs.len().to_string(),
            Some("text-blue-600".to_string()),
        ),
        (
            "Rules".to_string(),
            pool_ref.rules.len().to_string(),
            Some("text-green-600".to_string()),
        ),
        (
            "Rulesets".to_string(),
            pool_ref.rulesets.len().to_string(),
            Some("text-purple-600".to_string()),
        ),
    ];

    rsx! {
        Card {
            title: Some("Pool Status".to_string()),
            class: "bg-gray-50 rounded-lg p-4".to_string(),
            StatsGrid { stats: stats }
        }
    }
}

use layout::{
    core::base::Orientation,
    core::color::Color,
    core::geometry::Point,
    core::style::StyleAttr,
    std_shapes::shapes::{Arrow, Element as LayoutElement, ShapeKind},
    topo::layout::VisualGraph,
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct NodeStyle {
    pub fill_color: Option<Color>,
    pub line_color: Color,
    pub line_width: usize,
    pub font_size: usize,
    pub rounded: usize,
    pub min_width: f64,
    pub height: f64,
}

impl PartialEq for NodeStyle {
    fn eq(&self, other: &Self) -> bool {
        self.line_width == other.line_width
            && self.font_size == other.font_size
            && self.rounded == other.rounded
            && self.min_width == other.min_width
            && self.height == other.height
    }
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            fill_color: Some(Color::new(0xF5F5F5)),
            line_color: Color::new(0x757575),
            line_width: 1,
            font_size: 14,
            rounded: 5,
            min_width: 100.0,
            height: 40.0,
        }
    }
}

impl NodeStyle {
    pub fn primary() -> Self {
        Self {
            fill_color: Some(Color::new(0xE3F2FD)),
            line_color: Color::new(0x1976D2),
            line_width: 3,
            font_size: 16,
            ..Default::default()
        }
    }

    pub fn hover() -> Self {
        Self {
            fill_color: Some(Color::new(0xFFF9C4)),
            line_color: Color::new(0xF57C00),
            line_width: 2,
            font_size: 14,
            ..Default::default()
        }
    }

    pub fn child_by_depth(depth: usize) -> Self {
        let (fill_color, line_color) = match depth % 3 {
            0 => (0xF3E5F5, 0x7B1FA2),
            1 => (0xF1F8E9, 0x689F38),
            _ => (0xFFF3E0, 0xF57C00),
        };
        Self {
            fill_color: Some(Color::new(fill_color)),
            line_color: Color::new(line_color),
            font_size: 14 - depth.min(4),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct GraphNode<T: Clone> {
    pub id: T,
    pub label: String,
    pub style: NodeStyle,
    pub properties: Option<String>,
}

impl<T: Clone + PartialEq> PartialEq for GraphNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.label == other.label
            && self.style == other.style
            && self.properties == other.properties
    }
}

#[derive(Clone, Debug)]
pub struct GraphEdge<T: Clone> {
    pub from: T,
    pub to: T,
    pub label: Option<String>,
}

impl<T: Clone + PartialEq> PartialEq for GraphEdge<T> {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to && self.label == other.label
    }
}

#[component]
pub fn Graph<T: Clone + std::hash::Hash + Eq + std::fmt::Display + 'static>(
    nodes: Vec<GraphNode<T>>,
    edges: Vec<GraphEdge<T>>,
    #[props(default = false)] vertical: bool,
    #[props(default = true)] show_header: bool,
    #[props(default = "Graph".to_string())] title: String,
    on_node_click: Option<EventHandler<T>>,
    on_node_hover: Option<EventHandler<T>>,
    empty_message: Option<String>,
) -> Element {
    let mut tooltip_content = use_signal(|| None::<(String, Point)>);

    let orientation = if vertical {
        Orientation::TopToBottom
    } else {
        Orientation::LeftToRight
    };
    let mut vg = VisualGraph::new(orientation);
    let mut node_handles = HashMap::new();

    for node in &nodes {
        let shape = ShapeKind::new_box(&node.label);
        let mut look = StyleAttr::simple();

        look.fill_color = node.style.fill_color;
        look.line_color = node.style.line_color;
        look.line_width = node.style.line_width;
        look.font_size = node.style.font_size;
        look.rounded = node.style.rounded;

        let width = (node.label.len() as f64 * 8.0 + 20.0).max(node.style.min_width);
        let size = Point::new(width, node.style.height);

        let elem = if let Some(ref props) = node.properties {
            LayoutElement::create_with_properties(shape, look, orientation, size, props.clone())
        } else {
            LayoutElement::create(shape, look, orientation, size)
        };

        let handle = vg.add_node(elem);
        node_handles.insert(node.id.clone(), handle);
    }

    for edge in &edges {
        if let (Some(&from_handle), Some(&to_handle)) =
            (node_handles.get(&edge.from), node_handles.get(&edge.to))
        {
            let arrow = if let Some(ref label) = edge.label {
                Arrow::simple(label)
            } else {
                Arrow::simple("")
            };
            vg.add_edge(arrow, from_handle, to_handle);
        }
    }

    let mut svg_writer = super::svg_writer::DioxusSVGWriter::new();
    if !node_handles.is_empty() {
        vg.do_it(false, false, false, &mut svg_writer);
    }

    let node_positions = svg_writer.get_node_positions();

    rsx! {
        div { class: "h-full flex flex-col",
            if show_header {
                div { class: "flex-shrink-0 p-4 bg-gray-50 border-b border-gray-200",
                    div { class: "flex items-center justify-between",
                        h3 { class: "text-lg font-semibold text-gray-800", "{title}" }
                        div { class: "text-sm text-gray-500",
                            "Nodes: {nodes.len()}, Edges: {edges.len()}"
                        }
                    }
                }
            }

            div { class: "flex-1 overflow-auto relative",
                if node_handles.is_empty() {
                    div { class: "h-full flex items-center justify-center text-gray-500",
                        {empty_message.unwrap_or_else(|| "No data to display".to_string())}
                    }
                } else {
                    div {
                        class: "p-4",
                        onmousemove: move |e: Event<MouseData>| {
                            if on_node_hover.is_some() {
                                let pos = Point::new(
                                    e.client_coordinates().x,
                                    e.client_coordinates().y
                                );
                                tooltip_content.set(Some(("Hover info".to_string(), pos)));
                            }
                        },
                        onmouseleave: move |_| {
                            tooltip_content.set(None);
                        },
                        onclick: move |e: Event<MouseData>| {
                            if let Some(handler) = &on_node_click {
                                let click_x = e.element_coordinates().x;
                                let click_y = e.element_coordinates().y;

                                for (expr_id_str, (pos, size)) in &node_positions {
                                    if click_x >= pos.x && click_x <= pos.x + size.x &&
                                       click_y >= pos.y && click_y <= pos.y + size.y {
                                        for node in &nodes {
                                            if format!("{}", node.id) == *expr_id_str {
                                                handler.call(node.id.clone());
                                                break;
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        },
                        {svg_writer.render_svg()}
                    }

                    if let Some((text, pos)) = tooltip_content.read().as_ref() {
                        div {
                            class: "fixed bg-gray-900 text-white p-2 rounded shadow-lg text-sm pointer-events-none z-50",
                            style: "left: {pos.x + 10.0}px; top: {pos.y - 30.0}px;",
                            "{text}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ExpressionTreeGraph(
    pool: Signal<crate::Pool>,
    expr_id: crate::ExprId,
    #[props(default = false)] mini: bool,
) -> Element {
    let pool_ref = pool.read();
    let mut nodes = vec![];
    let mut edges = vec![];
    let mut visited = std::collections::HashSet::new();
    let mut stack = vec![(expr_id, 0)];

    while let Some((current_expr, depth)) = stack.pop() {
        if !visited.insert(current_expr) {
            continue;
        }

        let label = pool_ref.display_with_children(current_expr);
        let style = if current_expr == expr_id {
            NodeStyle::primary()
        } else {
            NodeStyle::child_by_depth(depth)
        };

        nodes.push(GraphNode {
            id: current_expr,
            label,
            style,
            properties: Some(format!("data-expr-id='{}'", current_expr.0)),
        });

        let children: Vec<_> = pool_ref.children(current_expr).collect();
        for (i, child_id) in children.iter().enumerate() {
            edges.push(GraphEdge {
                from: current_expr,
                to: *child_id,
                label: if mini { None } else { Some(format!("{}", i)) },
            });
            stack.push((*child_id, depth + 1));
        }
    }

    let title = if mini {
        "Expression Tree".to_string()
    } else {
        format!(
            "Expression Tree: {}",
            pool_ref.display_with_children(expr_id)
        )
    };

    rsx! {
        Graph {
            nodes: nodes,
            edges: edges,
            vertical: true,
            title: title,
            show_header: !mini,
            empty_message: Some("No expression structure to display".to_string()),
            on_node_click: None,
            on_node_hover: None,
        }
    }
}

#[component]
pub fn TransformationGraph(
    pool: Signal<crate::Pool>,
    current_expr: Option<crate::ExprId>,
    #[props(default = None)] hovered_expr: Option<crate::ExprId>,
    #[props(default = None)] on_node_click: Option<EventHandler<crate::ExprId>>,
) -> Element {
    if let Some(expr_id) = current_expr {
        let pool_ref = pool.read();
        let mut nodes = vec![];
        let mut edges = vec![];
        let mut all_expressions = std::collections::HashSet::new();

        for &source_expr in pool_ref.outgoing.keys() {
            all_expressions.insert(source_expr);
        }
        for targets in pool_ref.outgoing.values() {
            for &(target_expr, _) in targets {
                all_expressions.insert(target_expr);
            }
        }

        for &expr in &all_expressions {
            let style = if expr == expr_id {
                NodeStyle::primary()
            } else if Some(expr) == hovered_expr {
                NodeStyle::hover()
            } else {
                NodeStyle::default()
            };

            nodes.push(GraphNode {
                id: expr,
                label: pool_ref.display_with_children(expr),
                style,
                properties: Some(format!("data-expr-id='{}'", expr.0)),
            });
        }

        for &source_expr in &all_expressions {
            if let Some(outgoing) = pool_ref.get_outgoing_transformations(source_expr) {
                for &(target_expr, rule_id) in outgoing {
                    let rule_name = pool_ref.display_name(pool_ref[rule_id].name);
                    edges.push(GraphEdge {
                        from: source_expr,
                        to: target_expr,
                        label: Some(rule_name),
                    });
                }
            }
        }

        rsx! {
            Graph {
                nodes: nodes,
                edges: edges,
                vertical: true,
                title: "Transformation Graph".to_string(),
                show_header: true,
                empty_message: Some("No transformations found. Apply rules to see the graph.".to_string()),
                on_node_click: on_node_click,
                on_node_hover: None,
            }
        }
    } else {
        rsx! {
            div { class: "h-full flex items-center justify-center text-gray-500",
                "No expression selected for visualization"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UIError {
    ParseError {
        message: String,
        position: Option<usize>,
    },
    RuleApplicationError {
        rule_name: String,
        message: String,
    },
    GraphRenderError {
        message: String,
    },
    ValidationError {
        field: String,
        message: String,
    },
    NetworkError {
        message: String,
    },
    UnknownError {
        message: String,
    },
}

impl std::fmt::Display for UIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UIError::ParseError { message, position } => {
                if let Some(pos) = position {
                    write!(f, "Parse error at position {}: {}", pos, message)
                } else {
                    write!(f, "Parse error: {}", message)
                }
            }
            UIError::RuleApplicationError { rule_name, message } => {
                write!(f, "Rule '{}' failed: {}", rule_name, message)
            }
            UIError::GraphRenderError { message } => {
                write!(f, "Graph rendering error: {}", message)
            }
            UIError::ValidationError { field, message } => {
                write!(f, "{}: {}", field, message)
            }
            UIError::NetworkError { message } => {
                write!(f, "Network error: {}", message)
            }
            UIError::UnknownError { message } => {
                write!(f, "Error: {}", message)
            }
        }
    }
}

impl UIError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            UIError::ParseError { .. } => ErrorSeverity::Warning,
            UIError::RuleApplicationError { .. } => ErrorSeverity::Warning,
            UIError::GraphRenderError { .. } => ErrorSeverity::Error,
            UIError::ValidationError { .. } => ErrorSeverity::Warning,
            UIError::NetworkError { .. } => ErrorSeverity::Error,
            UIError::UnknownError { .. } => ErrorSeverity::Error,
        }
    }

    pub fn is_recoverable(&self) -> bool {
        !matches!(self, UIError::UnknownError { .. })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl ErrorSeverity {
    pub fn color_class(&self) -> &'static str {
        match self {
            ErrorSeverity::Info => "text-blue-600 bg-blue-50 border-blue-200",
            ErrorSeverity::Warning => "text-yellow-600 bg-yellow-50 border-yellow-200",
            ErrorSeverity::Error => "text-red-600 bg-red-50 border-red-200",
            ErrorSeverity::Critical => "text-red-800 bg-red-100 border-red-300",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ErrorSeverity::Info => "ℹ️",
            ErrorSeverity::Warning => "⚠️",
            ErrorSeverity::Error => "❌",
            ErrorSeverity::Critical => "🔥",
        }
    }
}

#[component]
pub fn ErrorBoundary(
    #[props(default = "Something went wrong".to_string())] fallback_message: String,
    #[props(default = true)] show_details: bool,
    children: Element,
) -> Element {
    let mut error_state = use_signal(|| None::<String>);

    if let Some(error) = error_state.read().as_ref() {
        rsx! {
            ErrorDisplay {
                error: UIError::UnknownError { message: error.clone() },
                show_details: show_details,
                on_retry: Some(EventHandler::new(move |_| {
                    error_state.set(None);
                })),
            }
        }
    } else {
        children
    }
}

#[component]
pub fn ErrorDisplay(
    error: UIError,
    #[props(default = true)] show_details: bool,
    #[props(default = true)] show_retry: bool,
    on_retry: Option<EventHandler<()>>,
    on_dismiss: Option<EventHandler<()>>,
) -> Element {
    let severity = error.severity();
    let is_recoverable = error.is_recoverable();

    rsx! {
        div { class: "border rounded-lg p-4 {severity.color_class()}",
            div { class: "flex items-start justify-between",
                div { class: "flex items-start space-x-3",
                    div { class: "text-lg flex-shrink-0", "{severity.icon()}" }
                    div { class: "flex-1",
                        div { class: "font-semibold text-sm mb-1",
                            match severity {
                                ErrorSeverity::Info => "Information",
                                ErrorSeverity::Warning => "Warning",
                                ErrorSeverity::Error => "Error",
                                ErrorSeverity::Critical => "Critical Error",
                            }
                        }
                        div { class: "text-sm", "{error}" }

                        if show_details {
                            details { class: "mt-2",
                                summary { class: "text-xs cursor-pointer hover:underline", "Technical Details" }
                                div { class: "mt-1 text-xs font-mono bg-black bg-opacity-10 p-2 rounded",
                                    "{error:?}"
                                }
                            }
                        }

                        if is_recoverable && (show_retry || on_dismiss.is_some()) {
                            div { class: "flex gap-2 mt-3",
                                if show_retry && on_retry.is_some() {
                                    button {
                                        class: "text-xs px-3 py-1 bg-white border border-current rounded hover:bg-opacity-10",
                                        onclick: move |_| {
                                            if let Some(handler) = &on_retry {
                                                handler.call(());
                                            }
                                        },
                                        "Retry"
                                    }
                                }
                                if let Some(dismiss_handler) = on_dismiss {
                                    button {
                                        class: "text-xs px-3 py-1 bg-white border border-current rounded hover:bg-opacity-10",
                                        onclick: move |_| dismiss_handler.call(()),
                                        "Dismiss"
                                    }
                                }
                            }
                        }
                    }
                }

                if on_dismiss.is_some() {
                    button {
                        class: "text-lg hover:opacity-70 flex-shrink-0",
                        onclick: move |_| {
                            if let Some(handler) = &on_dismiss {
                                handler.call(());
                            }
                        },
                        "×"
                    }
                }
            }
        }
    }
}

pub fn safe_operation<T, F>(operation: F, error_message: &str) -> Result<T, UIError>
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    std::panic::catch_unwind(operation).map_err(|_| UIError::UnknownError {
        message: format!("Operation panicked: {}", error_message),
    })
}

impl From<String> for UIError {
    fn from(error: String) -> Self {
        if error.contains("parse") || error.contains("Parse") {
            UIError::ParseError {
                message: error,
                position: None,
            }
        } else {
            UIError::UnknownError { message: error }
        }
    }
}

impl From<&str> for UIError {
    fn from(error: &str) -> Self {
        UIError::from(error.to_string())
    }
}

#[component]
pub fn Card(
    #[props(default = None)] title: Option<String>,
    #[props(default = "bg-white border border-gray-200 rounded-lg p-4".to_string())] class: String,
    children: Element,
) -> Element {
    rsx! {
        div { class: "{class}",
            if let Some(title) = title {
                div { class: "mb-3 pb-3 border-b border-gray-100",
                    h3 { class: "text-lg font-semibold text-gray-800", "{title}" }
                }
            }
            {children}
        }
    }
}

#[component]
pub fn InfoBox(
    #[props(default = "Info".to_string())] title: String,
    #[props(default = "bg-blue-50 border-blue-200 text-blue-800".to_string())]
    variant_class: String,
    children: Element,
) -> Element {
    rsx! {
        div { class: "border rounded-lg p-4 {variant_class}",
            div { class: "font-semibold text-sm mb-2", "{title}" }
            {children}
        }
    }
}

#[component]
pub fn Stat(
    label: String,
    value: String,
    #[props(default = "text-blue-600".to_string())] color_class: String,
) -> Element {
    rsx! {
        div { class: "bg-white rounded p-3 text-center shadow-sm",
            div { class: "text-2xl font-bold {color_class}", "{value}" }
            div { class: "text-sm text-gray-600", "{label}" }
        }
    }
}

#[component]
pub fn StatsGrid(stats: Vec<(String, String, Option<String>)>) -> Element {
    rsx! {
        div { class: "grid grid-cols-3 gap-4",
            for (label, value, color) in stats {
                Stat {
                    label: label,
                    value: value,
                    color_class: color.unwrap_or_else(|| "text-blue-600".to_string()),
                }
            }
        }
    }
}

#[component]
pub fn Badge(
    text: String,
    #[props(default = "bg-gray-100 text-gray-800".to_string())] variant_class: String,
) -> Element {
    rsx! {
        span { class: "inline-flex items-center px-2 py-1 rounded text-xs font-medium {variant_class}",
            "{text}"
        }
    }
}

#[component]
pub fn Divider(#[props(default = None)] text: Option<String>) -> Element {
    if let Some(text) = text {
        rsx! {
            div { class: "relative",
                div { class: "absolute inset-0 flex items-center",
                    div { class: "w-full border-t border-gray-200" }
                }
                div { class: "relative flex justify-center text-sm",
                    span { class: "px-2 bg-white text-gray-500", "{text}" }
                }
            }
        }
    } else {
        rsx! {
            div { class: "border-t border-gray-200" }
        }
    }
}

#[component]
pub fn TwoColumnLayout(
    left_title: String,
    right_title: String,
    left_content: Element,
    right_content: Element,
) -> Element {
    rsx! {
        div { class: "grid grid-cols-2 gap-6",
            div { class: "bg-white rounded-lg shadow p-6",
                h2 { class: "text-xl font-semibold mb-4", "{left_title}" }
                {left_content}
            }

            div { class: "bg-white rounded-lg shadow p-6",
                h2 { class: "text-xl font-semibold mb-4", "{right_title}" }
                {right_content}
            }
        }
    }
}

#[component]
pub fn WorkflowStep(
    step_number: usize,
    title: String,
    description: String,
    is_active: bool,
    is_completed: bool,
    content: Element,
) -> Element {
    let step_class = if is_completed {
        "bg-green-500 text-white"
    } else if is_active {
        "bg-blue-500 text-white"
    } else {
        "bg-gray-300 text-gray-600"
    };

    rsx! {
        div { class: "space-y-4",
            div { class: "flex items-center space-x-4",
                div { class: "w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold {step_class}",
                    "{step_number}"
                }
                div {
                    h3 { class: "text-lg font-semibold", "{title}" }
                    p { class: "text-sm text-gray-600", "{description}" }
                }
            }

            if is_active || is_completed {
                div { class: "ml-12 bg-gray-50 rounded-lg p-4",
                    {content}
                }
            }
        }
    }
}
