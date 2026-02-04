use hyprbar::prelude::*;

pub struct TextArea {
    instance_name: Option<String>,
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            instance_name: None,
        }
    }
}

impl Widget for TextArea {
    fn name(&self) -> &str {
        "text_area"
    }

    fn set_instance_config(&mut self, name: String) {
        self.instance_name = Some(name);
    }

    fn width(&self, state: &BarState) -> u16 {
        let content = self.get_content(state);
        content.chars().count() as u16
    }

    fn update(&mut self, _dt: std::time::Duration, _state: &BarState) {}

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        let content = self.get_content(state);

        Label::new(&content)
            .variant(TypographyVariant::Body)
            .render(area, buf, state.config_ink.as_ref());
    }
}

impl TextArea {
    fn get_content(&self, state: &BarState) -> String {
        let base_config = state
            .config
            .widget
            .get("text_area")
            .and_then(|v| v.as_table());

        // 1. Try instance config: [widget.text_area.alias].content
        if let Some(alias) = &self.instance_name {
            if let Some(content) = base_config
                .and_then(|t| t.get(alias))
                .and_then(|v| v.get("content"))
                .and_then(|v| v.as_str())
            {
                return content.to_string();
            }
        }

        // 2. Fallback to base config: [widget.text_area].content
        base_config
            .and_then(|t| t.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("hyprbar")
            .to_string()
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_widget() -> Box<dyn Widget> {
    Box::new(TextArea::new())
}

#[unsafe(no_mangle)]
pub extern "C" fn _plugin_metadata() -> *const std::ffi::c_char {
    static META: &[u8] =
        b"{\"author\":\"\",\"description\":\"\",\"name\":\"Unknown\",\"version\":\"0.0.1\"}\0";
    META.as_ptr() as *const _
}
