use super::BarRenderer;
use super::types::{ActivePopup, WidgetSection};
use crate::modules::logging::*;
use crate::state::BarState;
use crate::widget::PopupRequest;
use ratatui::prelude::*;

impl BarRenderer {
    pub fn check_popup_request(&self) -> Option<(PopupRequest, ActivePopup)> {
        // Active popup takes priority — avoids flickering if cursor briefly crosses another widget.
        if let Some(active) = &self.active_popup {
            let widget = match active.section {
                WidgetSection::Left => self.left_widgets.get(active.index),
                WidgetSection::Center => self.center_widgets.get(active.index),
                WidgetSection::Right => self.right_widgets.get(active.index),
            };
            if let Some(w) = widget
                && let Some(request) = w.popup_request()
            {
                if request.persist {
                    return Some((request, *active));
                }
                if self.hovered_widget == Some((active.section, active.index)) {
                    return Some((request, *active));
                }
            }
            return None;
        }

        let (section, idx) = self.hovered_widget?;

        let widget = match section {
            WidgetSection::Left => self.left_widgets.get(idx)?,
            WidgetSection::Center => self.center_widgets.get(idx)?,
            WidgetSection::Right => self.right_widgets.get(idx)?,
        };

        let request = widget.popup_request()?;

        let widget_area = self
            .hit_map
            .iter()
            .find(|h| h.section == section && h.index == idx)
            .map(|h| h.area)?;

        Some((
            request,
            ActivePopup {
                section,
                index: idx,
                widget_area,
            },
        ))
    }

    pub fn active_popup(&self) -> Option<&ActivePopup> {
        self.active_popup.as_ref()
    }

    pub fn set_active_popup(&mut self, popup: ActivePopup, width: u16, height: u16) {
        log_info(
            "POPUP",
            &format!(
                "Activating popup: section={:?}, index={}, size={}x{}",
                popup.section, popup.index, width, height
            ),
        );
        self.active_popup = Some(popup);
        self.popup_buffer = Some(Buffer::empty(Rect::new(0, 0, width, height)));
        log_debug(
            "POPUP",
            &format!("Popup buffer created {}x{}", width, height),
        );
    }

    pub fn clear_active_popup(&mut self) {
        if self.active_popup.is_some() {
            log_info("POPUP", "Deactivating popup");
        }
        self.active_popup = None;
        self.popup_buffer = None;
    }

    pub fn render_popup(&mut self, state: &BarState) -> Option<&Buffer> {
        let popup = self.active_popup?;
        let buf = self.popup_buffer.as_mut()?;

        buf.reset();
        let area = buf.area;

        let widget = match popup.section {
            WidgetSection::Left => self.left_widgets.get_mut(popup.index)?,
            WidgetSection::Center => self.center_widgets.get_mut(popup.index)?,
            WidgetSection::Right => self.right_widgets.get_mut(popup.index)?,
        };

        widget.render_popup(area, buf, state);

        self.popup_buffer.as_ref()
    }

    pub fn popup_buffer(&self) -> Option<&Buffer> {
        self.popup_buffer.as_ref()
    }
}
