//! Name: Interaction Demo
//! Version: 1.0.0
//! Author: Ryu
//! Description: Demo showing click, hover, and scroll handling
//! Has-Popup: true

use hyprbar::prelude::*;
use std::sync::Mutex;

pub struct InteractionDemoWidget {
    state: Mutex<InteractionState>,
}

struct InteractionState {
    hover: bool,
    click_count: u32,
    last_button: Option<u32>,
    scroll_offset: i32,
    show_popup: bool,
}

impl InteractionDemoWidget {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(InteractionState {
                hover: false,
                click_count: 0,
                last_button: None,
                scroll_offset: 0,
                show_popup: false,
            }),
        }
    }
}

impl Widget for InteractionDemoWidget {
    fn name(&self) -> &str {
        "interaction_demo"
    }

    fn width(&self, _state: &BarState) -> u16 {
        25
    }

    fn handle_event(&mut self, event: WidgetEvent) {
        let mut state = self.state.lock().unwrap();

        match event {
            WidgetEvent::Enter => {
                state.hover = true;
            }
            WidgetEvent::Leave => {
                state.hover = false;
            }
            WidgetEvent::Click { button, .. } => {
                state.click_count += 1;
                state.last_button = Some(button);
                // Toggle popup on left click
                if button == 272 {
                    state.show_popup = !state.show_popup;
                }
            }
            WidgetEvent::Scroll { dy, .. } => {
                state.scroll_offset += dy as i32;
                // Clamp to reasonable range
                state.scroll_offset = state.scroll_offset.clamp(-99, 99);
            }
            WidgetEvent::Motion { .. } => {
                // Could track position if needed
            }
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, _dt: Duration) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let interaction = self.state.lock().unwrap();

        // Display adapts to interaction history so users see feedback from their actions.
        let text = if interaction.hover {
            if let Some(btn) = interaction.last_button {
                // Wayland button codes: BTN_LEFT=272, BTN_RIGHT=273, BTN_MIDDLE=274
                let btn_name = match btn {
                    272 => "L",
                    273 => "R",
                    274 => "M",
                    _ => "?",
                };
                format!(
                    "[{}] x{} s:{}",
                    btn_name, interaction.click_count, interaction.scroll_offset
                )
            } else {
                format!("HOVER s:{}", interaction.scroll_offset)
            }
        } else if interaction.click_count > 0 {
            format!(
                "x{} s:{}",
                interaction.click_count, interaction.scroll_offset
            )
        } else {
            "Click/Scroll me".to_string()
        };

        // Accent variant on hover gives visual confirmation that the widget is interactive.
        let variant = if interaction.hover {
            TypographyVariant::Accent
        } else {
            TypographyVariant::Body
        };

        Label::new(&text)
            .variant(variant)
            .render(area, buf, state.config_ink.as_ref());
    }

    fn popup_request(&self) -> Option<PopupRequest> {
        let state = self.state.lock().unwrap();
        if state.show_popup {
            Some(PopupRequest::new(20, 5))
        } else {
            None
        }
    }

    fn render_popup(&mut self, area: Rect, buf: &mut Buffer, state: &BarState) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let interaction = self.state.lock().unwrap();

        // Popup shows accumulated stats so users can verify all input types are working.
        let lines = [
            format!("Clicks: {}", interaction.click_count),
            format!("Scroll: {}", interaction.scroll_offset),
            "Click to close".to_string(),
        ];

        for (i, line) in lines.iter().enumerate() {
            if i as u16 >= area.height {
                break;
            }
            let y = area.y + i as u16;
            Label::new(line)
                .variant(TypographyVariant::Body)
                .render(Rect::new(area.x, y, area.width, 1), buf, state.config_ink.as_ref());
        }
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_widget() -> Box<dyn Widget> {
    Box::new(InteractionDemoWidget::new())
}
