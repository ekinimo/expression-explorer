use crate::ui::file_utils::document::eval;
use dioxus::prelude::*;

/// Download a text file in the browser using JavaScript interop
pub fn download_text_file(filename: &str, content: &str) {
    let js_code = format!(
        r#"
        const blob = new Blob([{}], {{ type: 'text/plain' }});
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = {};
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        "#,
        serde_json::to_string(content).unwrap_or_default(),
        serde_json::to_string(filename).unwrap_or_default()
    );
    
    eval(&js_code);
}

/// Read a file from a FormEvent
pub async fn read_file_from_event(event: &FormEvent) -> Option<(String, String)> {
    if let Some(file_engine) = event.files() {
        let files = file_engine.files();
        if let Some(file_name) = files.first() {
            if let Some(contents) = file_engine.read_file_to_string(file_name).await {
                return Some((file_name.clone(), contents));
            }
        }
    }
    None
}
