use super::BarRenderer;
use super::types::{HitWidget, WidgetSection};
use crate::state::BarState;
use crate::widget::Widget;
use ratatui::prelude::*;
use std::time::Duration;

impl BarRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_section(
        buffer: &mut Buffer,
        area: Rect,
        widgets: &mut [Box<dyn Widget>],
        hit_map: &mut Vec<HitWidget>,
        section: WidgetSection,
        state: &BarState,
        align: Alignment,
        dt: Duration,
        padding: u16,
    ) {
        if widgets.is_empty() {
            return;
        }

        let widget_widths: Vec<u16> = widgets.iter().map(|d| d.width(state)).collect();
        let total_width: u16 = widget_widths.iter().sum();

        let mut current_x = match align {
            Alignment::Left => area.x,
            Alignment::Center => area.x + (area.width.saturating_sub(total_width)) / 2,
            Alignment::Right => area.x + area.width.saturating_sub(total_width),
        };

        for (i, widget) in widgets.iter_mut().enumerate() {
            let w = widget_widths[i];
            let render_area = Rect::new(current_x, area.y, w, area.height);
            let intersection = render_area.intersection(area);
            if !intersection.is_empty() {
                widget.render(intersection, buffer, state, dt);
                hit_map.push(HitWidget {
                    area: intersection,
                    section,
                    index: i,
                });
            }
            current_x += w + padding;
        }
    }

    pub(crate) fn calculate_flex_rects(
        &self,
        area: Rect,
        state: &BarState,
        padding: u16,
    ) -> Vec<Rect> {
        let left_w = Self::calc_width(&self.left_widgets, state, padding);
        let right_w = Self::calc_width(&self.right_widgets, state, padding);
        let center_w = Self::calc_width(&self.center_widgets, state, padding);

        let total_available = area.width;
        let final_left_w = left_w.min(total_available);
        let remaining_after_left = total_available.saturating_sub(final_left_w);
        let final_right_w = right_w.min(remaining_after_left);
        let remaining_for_center = remaining_after_left.saturating_sub(final_right_w);
        let final_center_w = center_w.min(remaining_for_center);

        let left_rect = Rect::new(area.x, area.y, final_left_w, area.height);
        let right_x = area.x + area.width - final_right_w;
        let right_rect = Rect::new(right_x, area.y, final_right_w, area.height);

        let ideal_center_x = area.x + (area.width.saturating_sub(final_center_w)) / 2;
        let min_center_x = left_rect.x + left_rect.width;
        let max_center_x = right_rect.x.saturating_sub(final_center_w);
        let final_center_x = ideal_center_x.clamp(min_center_x, max_center_x);
        let center_rect = Rect::new(final_center_x, area.y, final_center_w, area.height);

        vec![left_rect, center_rect, right_rect]
    }

    pub(crate) fn calc_width(widgets: &[Box<dyn Widget>], state: &BarState, padding: u16) -> u16 {
        if widgets.is_empty() {
            return 0;
        }
        let sum: u16 = widgets.iter().map(|d| d.width(state)).sum();
        let gaps = (widgets.len() as u16).saturating_sub(1);
        sum + gaps * padding
    }
}
