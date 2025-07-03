use dioxus::prelude::*;
use layout::core::{
    color::Color,
    format::{ClipHandle, RenderBackend},
    geometry::Point,
    style::StyleAttr,
};
use std::collections::HashMap;

pub struct DioxusSVGWriter {
    pub elements: Vec<VNode>,
    view_size: Point,
    counter: usize,
    font_style_map: HashMap<usize, String>,
    clip_regions: Vec<VNode>,
    node_positions: HashMap<String, (Point, Point)>, // expr_id -> (position, size)
    edge_label_positions: HashMap<String, (Point, Point)>, // edge_id -> (top-left, bottom-right)
}

impl Default for DioxusSVGWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl DioxusSVGWriter {
    pub fn new() -> Self {
        DioxusSVGWriter {
            elements: Vec::new(),
            view_size: Point::zero(),
            counter: 0,
            font_style_map: HashMap::new(),
            clip_regions: Vec::new(),
            node_positions: HashMap::new(),
            edge_label_positions: HashMap::new(),
        }
    }

    pub fn get_node_positions(&self) -> HashMap<String, (Point, Point)> {
        self.node_positions.clone()
    }

    pub fn get_edge_label_positions(&self) -> HashMap<String, (Point, Point)> {
        self.edge_label_positions.clone()
    }

    fn grow_window(&mut self, point: Point, size: Point) {
        self.view_size.x = self.view_size.x.max(point.x + size.x + 15.);
        self.view_size.y = self.view_size.y.max(point.y + size.y + 15.);
    }

    fn get_or_create_font_style(&mut self, font_size: usize) -> String {
        if let Some(class_name) = self.font_style_map.get(&font_size) {
            return class_name.clone();
        }
        let class_name = format!("font-{}px", font_size);
        self.font_style_map.insert(font_size, class_name.clone());
        class_name
    }

    pub fn get_viewbox(&self) -> String {
        format!("0 0 {} {}", self.view_size.x, self.view_size.y)
    }

    pub fn get_size(&self) -> (f64, f64) {
        (self.view_size.x, self.view_size.y)
    }

    pub fn render_svg(&self) -> Element {
        let (width, height) = self.get_size();


        let mut style_defs = String::new();
        for (size, class_name) in &self.font_style_map {
            style_defs += &format!(
                ".{} {{ font-size: {}px; font-family: Times, serif; }}\n",
                class_name, size
            );
        }


        style_defs += r#"
            .graph-node {
                cursor: pointer;
                transition: all 0.2s ease;
            }
            .graph-node:hover {
                filter: brightness(1.1);
                stroke-width: 3px !important;
            }
            .graph-edge-label {
                cursor: pointer;
                user-select: none;
            }
            .graph-edge-last-applied {
                stroke: #10b981 !important;
                stroke-width: 3px !important;
            }
        "#;

        rsx! {
            svg {
                width: "{width}",
                height: "{height}",
                view_box: "{self.get_viewbox()}",
                xmlns: "http://www.w3.org/2000/svg",

                rect {
                    width: "100%",
                    height: "100%",
                    fill: "white"
                }


                defs {
                    marker {
                        id: "startarrow",
                        marker_width: "10",
                        marker_height: "7",
                        ref_x: "0",
                        ref_y: "3.5",
                        orient: "auto",
                        polygon {
                            points: "10 0, 10 7, 0 3.5",
                            fill: "context-stroke"
                        }
                    }
                    marker {
                        id: "endarrow",
                        marker_width: "10",
                        marker_height: "7",
                        ref_x: "10",
                        ref_y: "3.5",
                        orient: "auto",
                        polygon {
                            points: "0 0, 10 3.5, 0 7",
                            fill: "context-stroke"
                        }
                    }
                }


                style { "{style_defs}" }


                for el in &self.clip_regions {
                    { el.clone() }
                }


                for el in &self.elements {
                    { el.clone() }
                }
            }
        }
    }
}

impl RenderBackend for DioxusSVGWriter {
    fn draw_rect(
        &mut self,
        xy: Point,
        size: Point,
        look: &StyleAttr,
        properties: Option<String>,
        clip: Option<ClipHandle>,
    ) {
        self.grow_window(xy, size);

        let fill = look
            .fill_color
            .unwrap_or_else(Color::transparent)
            .to_web_color();
        let stroke = look.line_color.to_web_color();
        let stroke_width = look.line_width;
        let rounded_px = look.rounded;
        let props = properties.unwrap_or_default();
        let clip_path = clip
            .map(|c| format!("clip-path: url(#C{c});"))
            .unwrap_or_default();

        // Check if this node has interaction data
        let has_node_data = props.contains("data-expr-id") || props.contains("data-rule-id");

        let rect_element = if has_node_data {
            // Extract node ID for position tracking
            let node_id = if props.contains("data-expr-id") {
                props
                    .split("data-expr-id='")
                    .nth(1)
                    .and_then(|s| s.split("'").next())
                    .map(|id| format!("expr_{}", id))
                    .unwrap_or_default()
            } else if props.contains("data-rule-id") {
                // Extract from, to, and rule id to reconstruct the GraphNodeId string representation
                let rule_id = props
                    .split("data-rule-id='")
                    .nth(1)
                    .and_then(|s| s.split("'").next())
                    .unwrap_or("");
                let from_id = props
                    .split("data-from='")
                    .nth(1)
                    .and_then(|s| s.split("'").next())
                    .unwrap_or("");
                let to_id = props
                    .split("data-to='")
                    .nth(1)
                    .and_then(|s| s.split("'").next())
                    .unwrap_or("");
                format!("rule_{}_{}_{}", from_id, to_id, rule_id)
            } else {
                String::new()
            };

            // Store node position for click detection
            if !node_id.is_empty() {
                self.node_positions.insert(node_id.clone(), (xy, size));
            }

            rsx!(rect {
                x: "{xy.x}",
                y: "{xy.y}",
                width: "{size.x}",
                height: "{size.y}",
                fill: "{fill}",
                stroke: "{stroke}",
                stroke_width: "{stroke_width}",
                rx: "{rounded_px}",
                style: "{clip_path} {props}",
                class: "graph-node clickable-node",
                "data-node-id": "{node_id}"
            })
        } else {
            rsx!(rect {
                x: "{xy.x}",
                y: "{xy.y}",
                width: "{size.x}",
                height: "{size.y}",
                fill: "{fill}",
                stroke: "{stroke}",
                stroke_width: "{stroke_width}",
                rx: "{rounded_px}",
                style: "{clip_path} {props}"
            })
        };

        self.elements.push(rect_element.unwrap());
    }

    fn draw_line(
        &mut self,
        start: Point,
        stop: Point,
        look: &StyleAttr,
        properties: Option<String>,
    ) {
        let stroke = look.line_color.to_web_color();
        let stroke_width = look.line_width;
        let props = properties.unwrap_or_default();

        self.elements.push(
            rsx!(line {
                x1: "{start.x}",
                y1: "{start.y}",
                x2: "{stop.x}",
                y2: "{stop.y}",
                stroke: "{stroke}",
                stroke_width: "{stroke_width}",
                style: "{props}"
            })
            .unwrap(),
        );
    }

    fn draw_circle(
        &mut self,
        xy: Point,
        size: Point,
        look: &StyleAttr,
        properties: Option<String>,
    ) {
        self.grow_window(xy, size);
        let fill = look
            .fill_color
            .unwrap_or_else(Color::transparent)
            .to_web_color();
        let stroke = look.line_color.to_web_color();
        let stroke_width = look.line_width;
        let props = properties.unwrap_or_default();

        self.elements.push(
            rsx!(ellipse {
                cx: "{xy.x}",
                cy: "{xy.y}",
                rx: "{size.x / 2.0}",
                ry: "{size.y / 2.0}",
                fill: "{fill}",
                stroke: "{stroke}",
                stroke_width: "{stroke_width}",
                style: "{props}"
            })
            .unwrap(),
        );
    }

    fn draw_text(&mut self, xy: Point, text: &str, look: &StyleAttr) {
        let class_name = self.get_or_create_font_style(look.font_size);

        self.grow_window(xy, Point::new(10., look.font_size as f64 * 1.5));

        self.elements.push(
            rsx!(text {
                x: "{xy.x}",
                y: "{xy.y}",
                class: "{class_name}",
                dominant_baseline: "middle",
                text_anchor: "middle",
                "{text}"
            })
            .unwrap(),
        );
    }

    fn draw_arrow(
        &mut self,
        path: &[(Point, Point)],
        dashed: bool,
        head: (bool, bool),
        look: &StyleAttr,
        properties: Option<String>,
        text: &str,
    ) {
        for (p0, p1) in path {
            self.grow_window(*p0, Point::zero());
            self.grow_window(*p1, Point::zero());
        }

        let stroke = look.line_color.to_web_color();
        let stroke_width = look.line_width;
        let dash = if dashed { "5,5" } else { "" };
        let marker_start = if head.0 { "url(#startarrow)" } else { "" };
        let marker_end = if head.1 { "url(#endarrow)" } else { "" };
        let props = properties.clone().unwrap_or_default();
        
        // Extract edge index from properties if available
        let mut edge_id_opt = None;
        if let Some(ref prop_str) = properties {
            if let Some(edge_idx_start) = prop_str.find("data-edge-index='") {
                let edge_idx_start = edge_idx_start + 17; // length of "data-edge-index='"
                if let Some(edge_idx_end) = prop_str[edge_idx_start..].find('\'') {
                    let edge_id = prop_str[edge_idx_start..edge_idx_start + edge_idx_end].to_string();
                    edge_id_opt = Some(edge_id);
                }
            }
        }

        let mut d = format!(
            "M {} {} C {} {}, {} {}, {} {} ",
            path[0].0.x,
            path[0].0.y,
            path[0].1.x,
            path[0].1.y,
            path[1].0.x,
            path[1].0.y,
            path[1].1.x,
            path[1].1.y
        );

        for (p0, p1) in path.iter().skip(2) {
            d += &format!("S {} {}, {} {} ", p0.x, p0.y, p1.x, p1.y);
        }

        let text_class = self.get_or_create_font_style(look.font_size);
        let id = self.counter;
        self.counter += 1;

        self.elements.push(
            rsx!(path {
                id: "arrow{id}",
                d: "{d}",
                stroke: "{stroke}",
                stroke_width: "{stroke_width}",
                stroke_dasharray: "{dash}",
                marker_start: "{marker_start}",
                marker_end: "{marker_end}",
                fill: "transparent",
                style: "{props}"
            })
            .unwrap(),
        );

        if !text.is_empty() {
            // Calculate label bounding box based on text length and font size
            if let (Some(edge_id), Some(first), Some(last)) = (edge_id_opt.as_ref(), path.first(), path.last()) {
                // Calculate the middle point of the path
                let mid_x = (first.0.x + last.1.x) / 2.0;
                let mid_y = (first.0.y + last.1.y) / 2.0;
                
                // Calculate text dimensions (approximate)
                let text_width = text.len() as f64 * (look.font_size as f64 * 0.6); // Approximate character width
                let text_height = look.font_size as f64 * 1.2; // Line height
                
                // Calculate bounding box
                let top_left = Point::new(mid_x - text_width / 2.0, mid_y - text_height / 2.0);
                let bottom_right = Point::new(mid_x + text_width / 2.0, mid_y + text_height / 2.0);
                
                self.edge_label_positions.insert(edge_id.clone(), (top_left, bottom_right));
            }
            
            self.elements.push(
                rsx!(text {
                    textPath {
                        href: "#arrow{id}",
                        start_offset: "50%",
                        text_anchor: "middle",
                        class: "{text_class} graph-edge-label",
                        dominant_baseline: "middle",
                        "{text}"
                    }
                })
                .unwrap(),
            );
        }
    }

    fn create_clip(&mut self, xy: Point, size: Point, rounded_px: usize) -> ClipHandle {
        let id = self.clip_regions.len();
        self.clip_regions.push(
            rsx!(
                clipPath {
                    id: "C{id}",
                    rect {
                        x: "{xy.x}",
                        y: "{xy.y}",
                        width: "{size.x}",
                        height: "{size.y}",
                        rx: "{rounded_px}"
                    }
                }
            )
            .unwrap(),
        );

        id
    }
}
