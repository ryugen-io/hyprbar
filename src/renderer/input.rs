use super::BarRenderer;
use super::types::WidgetSection;
use crate::event::WidgetEvent;
use crate::modules::logging::*;

impl BarRenderer {
    pub fn process_input(&mut self, x: u16, y: u16, event: WidgetEvent) {
        match &event {
            WidgetEvent::Enter => log_debug("INPUT", "Pointer entered bar"),
            WidgetEvent::Leave => log_debug("INPUT", "Pointer left bar"),
            _ => {}
        }

        if let WidgetEvent::Leave = event {
            if let Some((section, idx)) = self.hovered_widget {
                match section {
                    WidgetSection::Left => self.left_widgets[idx].handle_event(WidgetEvent::Leave),
                    WidgetSection::Center => {
                        self.center_widgets[idx].handle_event(WidgetEvent::Leave)
                    }
                    WidgetSection::Right => {
                        self.right_widgets[idx].handle_event(WidgetEvent::Leave)
                    }
                }
            }
            self.hovered_widget = None;
            return;
        }

        let mut hit_found = None;
        for hit in &self.hit_map {
            if x >= hit.area.x
                && x < hit.area.x + hit.area.width
                && y >= hit.area.y
                && y < hit.area.y + hit.area.height
            {
                hit_found = Some((hit.section, hit.index, hit.area));
                break;
            }
        }

        if let Some((section, idx, area)) = hit_found {
            if self.hovered_widget != Some((section, idx)) {
                if let Some((old_sec, old_idx)) = self.hovered_widget {
                    let old_name = match old_sec {
                        WidgetSection::Left => self.left_widgets[old_idx].name(),
                        WidgetSection::Center => self.center_widgets[old_idx].name(),
                        WidgetSection::Right => self.right_widgets[old_idx].name(),
                    };
                    log_debug("WIDGET", &format!("Leave: {}", old_name));
                    match old_sec {
                        WidgetSection::Left => {
                            self.left_widgets[old_idx].handle_event(WidgetEvent::Leave)
                        }
                        WidgetSection::Center => {
                            self.center_widgets[old_idx].handle_event(WidgetEvent::Leave)
                        }
                        WidgetSection::Right => {
                            self.right_widgets[old_idx].handle_event(WidgetEvent::Leave)
                        }
                    }
                }
                let new_name = match section {
                    WidgetSection::Left => self.left_widgets[idx].name(),
                    WidgetSection::Center => self.center_widgets[idx].name(),
                    WidgetSection::Right => self.right_widgets[idx].name(),
                };
                log_debug("WIDGET", &format!("Enter: {}", new_name));
                match section {
                    WidgetSection::Left => self.left_widgets[idx].handle_event(WidgetEvent::Enter),
                    WidgetSection::Center => {
                        self.center_widgets[idx].handle_event(WidgetEvent::Enter)
                    }
                    WidgetSection::Right => {
                        self.right_widgets[idx].handle_event(WidgetEvent::Enter)
                    }
                }
                self.hovered_widget = Some((section, idx));
            }

            let mut local_event = event;
            match &mut local_event {
                WidgetEvent::Motion { x: mx, y: my } | WidgetEvent::Click { x: mx, y: my, .. } => {
                    *mx = mx.saturating_sub(area.x);
                    *my = my.saturating_sub(area.y);
                }
                _ => {}
            }

            let widget_name = match section {
                WidgetSection::Left => self.left_widgets[idx].name(),
                WidgetSection::Center => self.center_widgets[idx].name(),
                WidgetSection::Right => self.right_widgets[idx].name(),
            };
            match &local_event {
                WidgetEvent::Click { button, .. } => {
                    let btn = match *button {
                        272 => "L",
                        273 => "R",
                        274 => "M",
                        _ => "?",
                    };
                    log_info("INPUT", &format!("Click {} on {}", btn, widget_name));
                }
                WidgetEvent::Scroll { dy, .. } => {
                    log_debug("INPUT", &format!("Scroll {} on {}", dy, widget_name));
                }
                _ => {}
            }
            match section {
                WidgetSection::Left => self.left_widgets[idx].handle_event(local_event),
                WidgetSection::Center => self.center_widgets[idx].handle_event(local_event),
                WidgetSection::Right => self.right_widgets[idx].handle_event(local_event),
            }
        } else {
            if let Some((old_sec, old_idx)) = self.hovered_widget {
                match old_sec {
                    WidgetSection::Left => {
                        self.left_widgets[old_idx].handle_event(WidgetEvent::Leave)
                    }
                    WidgetSection::Center => {
                        self.center_widgets[old_idx].handle_event(WidgetEvent::Leave)
                    }
                    WidgetSection::Right => {
                        self.right_widgets[old_idx].handle_event(WidgetEvent::Leave)
                    }
                }
            }
            self.hovered_widget = None;
        }
    }
}
