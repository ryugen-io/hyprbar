use crate::wayland::state::WaylandState;
use smithay_client_toolkit::reexports::client::protocol::wl_pointer;
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::{
    compositor::CompositorHandler,
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::client::{
        Connection, QueueHandle,
        protocol::{wl_output, wl_seat, wl_surface},
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::pointer::{PointerEvent, PointerEventKind, PointerHandler},
    seat::{Capability, SeatHandler, SeatState},
    shell::wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
    shm::{Shm, ShmHandler},
};

impl PointerHandler for WaylandState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use crate::event::WidgetEvent;
        for event in events {
            match event.kind {
                PointerEventKind::Enter { .. } => {
                    self.input_events.push(WidgetEvent::Enter);
                    self.input_events.push(WidgetEvent::Motion {
                        x: event.position.0 as u16,
                        y: event.position.1 as u16,
                    });
                    self.cursor_x = event.position.0;
                    self.cursor_y = event.position.1;
                    self.redraw_requested = true;
                }
                PointerEventKind::Leave { .. } => {
                    self.input_events.push(WidgetEvent::Leave);
                    self.redraw_requested = true;
                }
                PointerEventKind::Motion { .. } => {
                    self.input_events.push(WidgetEvent::Motion {
                        x: event.position.0 as u16,
                        y: event.position.1 as u16,
                    });
                    self.cursor_x = event.position.0;
                    self.cursor_y = event.position.1;
                    self.redraw_requested = true;
                }
                PointerEventKind::Press { button, .. } => {
                    self.input_events.push(WidgetEvent::Click {
                        button,
                        x: self.cursor_x as u16,
                        y: self.cursor_y as u16,
                    });
                    self.redraw_requested = true;
                }
                PointerEventKind::Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    if horizontal.absolute != 0.0 || vertical.absolute != 0.0 {
                        self.input_events.push(WidgetEvent::Scroll {
                            dx: horizontal.absolute,
                            dy: vertical.absolute,
                        });
                        self.redraw_requested = true;
                    }
                }
                _ => {}
            }
        }
    }
}

impl CompositorHandler for WaylandState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.redraw_requested = true;
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for WaylandState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for WaylandState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        // Check if it's the popup or main surface
        if let Some(popup_layer) = &self.popup_layer
            && popup_layer.wl_surface() == layer.wl_surface()
        {
            // Popup was closed externally
            hyprlog::internal::debug("POPUP", "Popup closed externally");
            self.popup_layer = None;
            self.popup_surface = None;
            self.popup_pool = None;
            self.popup_configured = false;
            return;
        }
        // Main surface closed
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        // Check if it's the popup or main surface
        if let Some(popup_layer) = &self.popup_layer
            && popup_layer.wl_surface() == layer.wl_surface()
        {
            // Popup configured
            if configure.new_size.0 != 0 && configure.new_size.1 != 0 {
                self.popup_width = configure.new_size.0;
                self.popup_height = configure.new_size.1;
            }
            self.popup_configured = true;
            self.popup_redraw_requested = true;
            hyprlog::internal::debug(
                "POPUP",
                &format!(
                    "Popup configured {}x{}",
                    self.popup_width, self.popup_height
                ),
            );
            return;
        }
        // Main surface configured
        if configure.new_size.0 != 0 && configure.new_size.1 != 0 {
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }
        self.configured = true;
    }
}

impl ShmHandler for WaylandState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl SeatHandler for WaylandState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.seat_state.get_pointer(qh, &seat).is_ok() {
            hyprlog::internal::debug("WAYLAND", "Got pointer capability");
        }
    }
    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _: Capability,
    ) {
    }
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

delegate_compositor!(WaylandState);
delegate_output!(WaylandState);
delegate_shm!(WaylandState);
delegate_seat!(WaylandState);
delegate_pointer!(WaylandState);
delegate_registry!(WaylandState);
delegate_layer!(WaylandState);

impl ProvidesRegistryState for WaylandState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState, SeatState];
}
