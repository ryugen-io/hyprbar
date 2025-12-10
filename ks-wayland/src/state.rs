use crate::blitter::blit_buffer_to_pixels;
use anyhow::Context;
use k_lib::config::Cookbook;
use ratatui::buffer::Buffer;
use smithay_client_toolkit::reexports::client::protocol::wl_shm;
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::client::{QueueHandle, protocol::wl_surface::WlSurface},
    registry::RegistryState,
    seat::SeatState,
    shell::wlr_layer::LayerShell,
    shm::{Shm, slot::SlotPool},
};

pub struct WaylandState {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub shm: Shm,
    pub layer_shell: LayerShell,

    pub pool: SlotPool,
    pub redraw_requested: bool,

    // Application state
    pub exit: bool,
    pub surface: Option<WlSurface>,
    pub configured: bool,
    pub width: u32,
    pub height: u32,
}

impl WaylandState {
    pub fn new(_globals: &RegistryState, _qh: &QueueHandle<Self>) -> Self {
        unimplemented!("Use helper initialization")
    }

    pub fn draw(
        &mut self,
        _qh: &QueueHandle<Self>,
        buffer: &Buffer,
        cookbook: &Cookbook,
    ) -> anyhow::Result<()> {
        let width = self.width;
        let height = self.height;

        if width == 0 || height == 0 {
            return Ok(());
        }

        let stride = width as i32 * 4;

        // Create buffer
        let (wl_buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .context("Failed to create buffer")?;

        // Blit
        blit_buffer_to_pixels(buffer, canvas, width, height, cookbook);

        // Attach and damage
        if let Some(surface) = &self.surface {
            // create_buffer returns (WlBuffer, &mut [u8])
            // WlBuffer is a Proxy.
            // surface.attach(Some(&wl_buffer), 0, 0);

            surface.attach(Some(wl_buffer.wl_buffer()), 0, 0);
            surface.damage_buffer(0, 0, width as i32, height as i32);
            surface.frame(_qh, surface.clone());
            surface.commit();
        }

        self.redraw_requested = false;
        Ok(())
    }
}
