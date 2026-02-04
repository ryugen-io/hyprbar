//! Name: UI Kit Demo
//! Version: 1.0.0
//! Author: Ryu
//! Description: Animated demo showing UI Kit typography variants

use hyprbar::prelude::*;
use std::sync::Mutex;
use tachyonfx::{Effect, EffectTimer, Interpolation, fx};

pub struct UiKitDemoWidget {
    effect: Mutex<Option<Effect>>,
}

impl UiKitDemoWidget {
    pub fn new() -> Self {
        Self {
            effect: Mutex::new(None),
        }
    }
}

impl Widget for UiKitDemoWidget {
    fn name(&self) -> &str {
        "ui_kit_demo"
    }

    fn width(&self, _state: &BarState) -> u16 {
        20
    }

    fn update(&mut self, _dt: Duration, state: &BarState) {
        let mut effect_lock = self.effect.lock().unwrap();
        if effect_lock.is_none() {
            let accent = state.config_ink.resolve_color("accent");
            // Smooth breathing: fade foreground from accent color over 2.5s, ping-pong loops forever
            *effect_lock = Some(fx::ping_pong(fx::fade_from_fg(
                accent,
                EffectTimer::from_ms(2500, Interpolation::SineInOut),
            )));
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &BarState, dt: Duration) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Render the label
        Label::new("UI Kit Demo")
            .variant(TypographyVariant::Accent)
            .render(area, buf, state.config_ink.as_ref());

        // Apply breathing effect only while running
        let mut effect_lock = self.effect.lock().unwrap();
        if let Some(effect) = effect_lock.as_mut() {
            if effect.running() {
                effect.process(dt, buf, area);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn _create_widget() -> Box<dyn Widget> {
    Box::new(UiKitDemoWidget::new())
}
