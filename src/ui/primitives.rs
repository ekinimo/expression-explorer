use crate::DisplayNode;
use dioxus::prelude::*;
use serde::{Serialize, Deserialize};

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
    std_shapes::shapes::{Arrow, Element as LayoutElement, ShapeKind, RecordDef},
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
    pub shape: Option<ShapeKind>,
    pub properties: Option<String>,
}

impl<T: Clone + PartialEq> PartialEq for GraphNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.label == other.label
            && self.style == other.style
            && self.properties == other.properties
        // Note: shape is not compared because ShapeKind doesn't implement PartialEq
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
    #[props(default = None)] last_applied_edge: Option<(T, T)>,
    on_node_click: Option<EventHandler<T>>,
    on_node_hover: Option<EventHandler<T>>,
    on_edge_hover: Option<EventHandler<Option<(usize, T, T)>>>,
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
        let shape = node.shape.clone().unwrap_or_else(|| ShapeKind::new_box(&node.label));
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

    for (i, edge) in edges.iter().enumerate() {
        if let (Some(&from_handle), Some(&to_handle)) =
            (node_handles.get(&edge.from), node_handles.get(&edge.to))
        {
            // Check if this is the last applied edge
            let is_last_applied = if let Some((last_from, last_to)) = &last_applied_edge {
                edge.from == *last_from && edge.to == *last_to
            } else {
                false
            };
            
            // Add properties for hover detection and styling
            let class_name = if is_last_applied {
                "graph-edge-last-applied"
            } else {
                ""
            };
            
            let properties = format!(
                "{}data-edge-index='{}' data-from='{}' data-to='{}'",
                if !class_name.is_empty() { format!("class='{}' ", class_name) } else { String::new() },
                i,
                edge.from,
                edge.to
            );
            let arrow = Arrow::simple_with_properties(edge.label.as_deref().unwrap_or(""), properties);
            vg.add_edge(arrow, from_handle, to_handle);
        }
    }

    let mut svg_writer = super::svg_writer::DioxusSVGWriter::new();
    if !node_handles.is_empty() {
        vg.do_it(false, false, false, &mut svg_writer);
    }

    let node_positions = svg_writer.get_node_positions();
    let edge_label_positions = svg_writer.get_edge_label_positions();

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
                            let mouse_x = e.element_coordinates().x;
                            let mouse_y = e.element_coordinates().y;
                            
                            // Check if hovering over an edge label
                            if let Some(handler) = &on_edge_hover {
                                let mut found_edge = false;
                                let _mouse_point = Point::new(mouse_x, mouse_y);
                                
                                // Check each edge label bounding box
                                for (edge_idx_str, (top_left, bottom_right)) in &edge_label_positions {
                                    if let Ok(edge_idx) = edge_idx_str.parse::<usize>() {
                                        // Check if mouse is within the label bounding box
                                        if mouse_x >= top_left.x && mouse_x <= bottom_right.x &&
                                           mouse_y >= top_left.y && mouse_y <= bottom_right.y {
                                            if let Some(edge) = edges.get(edge_idx) {
                                                handler.call(Some((edge_idx, edge.from.clone(), edge.to.clone())));
                                                found_edge = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                                
                                if !found_edge {
                                    handler.call(None);
                                }
                            }
                            
                            if on_node_hover.is_some() {
                                let pos = Point::new(
                                    e.client_coordinates().x,
                                    e.client_coordinates().y
                                );
                                tooltip_content.set(Some(("Hover info".to_string(), pos)));
                            }
                        },
                        onmouseleave: move |_| {
                            if let Some(handler) = &on_edge_hover {
                                handler.call(None);
                            }
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


// Node ID type that can represent either an expression or a rule
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum GraphNodeId {
    Expression(crate::ExprId),
    Rule(crate::ExprId, crate::ExprId, crate::RuleId), // from, to, rule
}

impl std::fmt::Display for GraphNodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphNodeId::Expression(id) => write!(f, "expr_{}", id.0),
            GraphNodeId::Rule(from, to, rule) => write!(f, "rule_{}_{}_{}", from.0, to.0, rule.0),
        }
    }
}

#[component]
pub fn TransformationGraph(
    pool: Signal<crate::Pool>,
    current_expr: Option<crate::ExprId>,
    #[props(default = None)] hovered_expr: Option<crate::ExprId>,
    #[props(default = None)] last_applied_rule: Option<(crate::ExprId, crate::ExprId, crate::RuleId, crate::rules::Match)>,
    #[props(default = None)] on_node_click: Option<EventHandler<crate::ExprId>>,
) -> Element {
    let _svg_content = use_signal(String::new);
    
    if let Some(expr_id) = current_expr {
        let pool_ref = pool.read();
        let mut nodes: Vec<GraphNode<GraphNodeId>> = vec![];
        let mut edges: Vec<GraphEdge<GraphNodeId>> = vec![];
        let mut group_to_representative = std::collections::HashMap::new();
        
        // Get current expression's equivalence group
        let current_group = pool_ref.get_equivalence_group(expr_id);
        
        // Collect all equivalence groups that have transformations
        let mut relevant_groups = std::collections::HashSet::new();
        
        // Add current expression's group
        if let Some(group) = current_group {
            relevant_groups.insert(group);
        }
        
        // Find all groups with outgoing transformations
        for (&from_group, targets) in &pool_ref.equivalence_outgoing {
            relevant_groups.insert(from_group);
            for &(to_group, _) in targets {
                relevant_groups.insert(to_group);
            }
        }
        
        // For each group, pick a representative expression and create a node
        for &group_id in &relevant_groups {
            if let Some(group_exprs) = pool_ref.get_group_expressions(group_id) {
                // Pick the first expression as representative, or the current expr if it's in this group
                let representative = if current_group == Some(group_id) {
                    expr_id
                } else {
                    *group_exprs.iter().next().unwrap()
                };
                
                group_to_representative.insert(group_id, representative);
                
                // Create label showing only the representative expression
                let label = pool_ref.display_with_children(representative);
                
                let mut style = if current_group == Some(group_id) {
                    NodeStyle::primary()
                } else if Some(representative) == hovered_expr {
                    NodeStyle::hover()
                } else {
                    NodeStyle::default()
                };
                
                // Make nodes more compact
                style.font_size = 12;
                style.height = 35.0;
                style.min_width = 80.0;
                
                nodes.push(GraphNode {
                    id: GraphNodeId::Expression(representative),
                    label,
                    style,
                    shape: None, // Use default box shape for expressions
                    properties: Some(format!("data-group-id='{}' data-expr-id='{}'", group_id.0, representative.0)),
                });
            }
        }
        
        // Create edges between groups and store metadata including captures
        let mut edge_metadata = std::collections::HashMap::new();
        for &from_group in &relevant_groups {
            if let Some(outgoing) = pool_ref.equivalence_outgoing.get(&from_group) {
                for &(to_group, rule_id) in outgoing {
                    if let (Some(&from_repr), Some(&to_repr)) = 
                        (group_to_representative.get(&from_group), group_to_representative.get(&to_group)) {
                        let _rule_name = pool_ref.display_name(pool_ref[rule_id].name);
                        
                        // Find the match that created this transformation to get captures
                        let matches = pool_ref.find_matches(from_repr);
                        let match_with_captures = matches.iter()
                            .find(|m| m.rule_id == rule_id)
                            .cloned();
                        
                        // Store metadata for hover functionality including captures
                        edge_metadata.insert((from_repr, to_repr), (rule_id, match_with_captures.clone()));
                        
                        // Check if this is the last applied rule
                        let is_last_applied = if let Some((last_from, last_to, last_rule_id, _)) = last_applied_rule {
                            from_repr == last_from && to_repr == last_to && rule_id == last_rule_id
                        } else {
                            false
                        };
                        
                        // Create intermediate rule node
                        let rule_node_id = GraphNodeId::Rule(from_repr, to_repr, rule_id);
                        let rule_name = pool_ref.display_name(pool_ref[rule_id].name);
                        
                        // Get rule pattern and action
                        let rule = pool_ref[rule_id];
                        let pattern_str = pool_ref.display_with_children(rule.pattern);
                        let action_str = pool_ref.display_with_children(rule.action);
                        
                        // Create nested record structure
                        let mut main_fields = vec![];
                        
                        // Rule name as header
                        main_fields.push(RecordDef::new_text(&format!("Rule: {}", rule_name)));
                        
                        // Pattern section
                        main_fields.push(RecordDef::Array(vec![
                            RecordDef::new_text("Pattern:"),
                            RecordDef::new_text(&pattern_str),
                        ]));
                        
                        // Action section
                        main_fields.push(RecordDef::Array(vec![
                            RecordDef::new_text("Action:"),
                            RecordDef::new_text(&action_str),
                        ]));
                        
                        // Captures section (if any)
                        if let Some(ref match_) = match_with_captures {
                            if !match_.captures.is_empty() {
                                let mut capture_records = vec![];
                                
                                // Create capture records with each capture as a horizontal array
                                for (name_id, captured_value) in &match_.captures {
                                    let var_name = format!("?{}", pool_ref[*name_id]);
                                    let value_str = match captured_value {
                                        crate::rules::CapturedValue::Expression(expr_id) => {
                                            pool_ref.display_with_children(*expr_id).to_string()
                                        }
                                        crate::rules::CapturedValue::Function(fun_id) => {
                                            format!("fn:{}", pool_ref[*fun_id])
                                        }
                                        crate::rules::CapturedValue::StructName(name_id_val) => {
                                            format!("struct:{}", pool_ref[*name_id_val])
                                        }
                                    };
                                    
                                    // Each capture is a horizontal array: [var, arrow, value]
                                    capture_records.push(RecordDef::Array(vec![
                                        RecordDef::new_text(&var_name),
                                        RecordDef::new_text("→"),
                                        RecordDef::new_text(&value_str),
                                    ]));
                                }
                                
                                // Captures section with header and nested capture records
                                main_fields.push(RecordDef::Array(vec![
                                    RecordDef::new_text("Captures:"),
                                    RecordDef::Array(capture_records),
                                ]));
                            }
                        }
                        
                        let shape = Some(ShapeKind::Record(RecordDef::Array(main_fields)));
                        let label = String::new(); // Empty label since everything is in the record
                        
                        // Calculate node height based on content
                        // Base height + pattern/action sections + captures if any
                        let capture_count = match_with_captures.as_ref().map_or(0, |m| m.captures.len());
                        let has_captures = capture_count > 0;
                        let node_height = 100.0 + (if has_captures { 20.0 + (capture_count as f64 * 16.0) } else { 0.0 });
                        
                        // Style for rule node - yellow theme
                        let rule_style = if is_last_applied {
                            NodeStyle {
                                fill_color: Some(Color::new(0xFBBF24)), // amber-400
                                line_color: Color::new(0xF59E0B), // amber-500
                                line_width: 3,
                                font_size: 10,
                                height: node_height,
                                min_width: 180.0,
                                rounded: 8,
                            }
                        } else {
                            NodeStyle {
                                fill_color: Some(Color::new(0xFEF3C7)), // amber-100
                                line_color: Color::new(0xF59E0B), // amber-500
                                line_width: 1,
                                font_size: 10,
                                height: node_height,
                                min_width: 180.0,
                                rounded: 8,
                            }
                        };
                        
                        // Add rule node
                        nodes.push(GraphNode {
                            id: rule_node_id.clone(),
                            label,
                            style: rule_style,
                            shape,
                            properties: Some(format!("data-rule-id='{}' data-from='{}' data-to='{}'", rule_id.0, from_repr.0, to_repr.0)),
                        });
                        
                        // Create edges: from_expr -> rule_node -> to_expr
                        edges.push(GraphEdge {
                            from: GraphNodeId::Expression(from_repr),
                            to: rule_node_id.clone(),
                            label: None,
                        });
                        
                        edges.push(GraphEdge {
                            from: rule_node_id,
                            to: GraphNodeId::Expression(to_repr),
                            label: None,
                        });
                    }
                }
            }
        }
        
        rsx! {
            div { class: "h-full w-full flex flex-col",
                // Export button and header
                div { class: "flex items-center justify-between p-4 border-b",
                    h3 { class: "text-lg font-semibold", "Equivalence Classes Graph" }
                    button {
                        class: format!("{} text-sm", crate::ui::styles::BTN_SECONDARY),
                        onclick: move |_| {
                            use layout::backends::svg::SVGWriter;
                            
                            // Generate SVG content
                            let mut vg = VisualGraph::new(Orientation::TopToBottom);
                            let mut node_handles = HashMap::new();
                            
                            // Add all nodes to the graph
                            for node in &nodes {
                                let shape = node.shape.clone().unwrap_or_else(|| ShapeKind::new_box(&node.label));
                                let mut look = StyleAttr::simple();
                                look.fill_color = node.style.fill_color;
                                look.line_color = node.style.line_color;
                                look.line_width = node.style.line_width;
                                look.font_size = node.style.font_size;
                                look.rounded = node.style.rounded;
                                
                                let width = (node.label.len() as f64 * 8.0 + 20.0).max(node.style.min_width);
                                let size = Point::new(width, node.style.height);
                                
                                let elem = if let Some(ref props) = node.properties {
                                    LayoutElement::create_with_properties(shape, look, Orientation::TopToBottom, size, props.clone())
                                } else {
                                    LayoutElement::create(shape, look, Orientation::TopToBottom, size)
                                };
                                
                                let handle = vg.add_node(elem);
                                node_handles.insert(node.id.clone(), handle);
                            }
                            
                            // Add all edges
                            for edge in &edges {
                                if let (Some(&from_handle), Some(&to_handle)) = 
                                    (node_handles.get(&edge.from), node_handles.get(&edge.to)) {
                                    let arrow = Arrow::simple(edge.label.as_deref().unwrap_or(""));
                                    vg.add_edge(arrow, from_handle, to_handle);
                                }
                            }
                            
                            // Render to SVG using the standard SVGWriter
                            if !node_handles.is_empty() {
                                let mut svg_writer = SVGWriter::new();
                                vg.do_it(false, false, false, &mut svg_writer);
                                let svg_content = svg_writer.finalize();
                                
                                crate::ui::file_utils::download_text_file("transformation_graph.svg", &svg_content);
                            }
                        },
                        "📊 Export SVG"
                    }
                }
                
                // Graph content
                div { class: "flex-1 overflow-auto p-4 relative",
                    Graph {
                        nodes: nodes.clone(),
                        edges: edges.clone(),
                        vertical: true,
                        title: "".to_string(), // Title already in header
                        show_header: false,
                        last_applied_edge: None,
                        empty_message: Some("No transformations found. Apply rules to see the graph.".to_string()),
                        on_node_click: {
                            on_node_click.map(|handler| EventHandler::new(move |node_id: GraphNodeId| {
                                    // Only handle clicks on expression nodes
                                    if let GraphNodeId::Expression(expr_id) = node_id {
                                        handler.call(expr_id);
                                    }
                                    // Rule nodes are just for display
                                }))
                        },
                        on_node_hover: None,
                        on_edge_hover: None,
                    }
                }
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
