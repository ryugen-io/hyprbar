pub mod blitter;
pub mod handlers;
pub mod state;
pub mod text;

use anyhow::{Context, Result};
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::client::{Connection, EventQueue, globals::registry_queue_init},
    registry::RegistryState,
    seat::SeatState,
    shell::wlr_layer::{Anchor, Layer, LayerShell, LayerSurface},
    shm::{Shm, slot::SlotPool},
};
use state::WaylandState;
use text::TextRenderer;

pub fn init(
    height: u32,
    anchor_bottom: bool,
    monitor: Option<String>,
    font_path: Option<String>,
    font_size: f32,
) -> Result<(WaylandState, EventQueue<WaylandState>, LayerSurface)> {
    let conn = Connection::connect_to_env().context("Failed to connect to Wayland")?;

    let (globals, mut event_queue) =
        registry_queue_init::<WaylandState>(&conn).context("Failed to init registry queue")?;
    let qh = event_queue.handle();

    let registry_state = RegistryState::new(&globals);

    let compositor_state =
        CompositorState::bind(&globals, &qh).context("Failed to bind compositor")?;
    let layer_shell = LayerShell::bind(&globals, &qh).context("Failed to bind layer shell")?;
    let shm = Shm::bind(&globals, &qh).context("Failed to bind shm")?;
    let output_state = OutputState::new(&globals, &qh);
    let seat_state = SeatState::new(&globals, &qh);

    let pool = SlotPool::new(1920 * 1080 * 4, &shm).context("Failed to create Shm pool")?;

    let text_renderer = TextRenderer::new(font_path.as_deref(), font_size)
        .context("Failed to initialize text renderer")?;

    let mut state = WaylandState {
        registry_state,
        seat_state,
        output_state,
        compositor_state,
        shm,
        layer_shell,
        pool,
        redraw_requested: true,
        exit: false,
        surface: None,
        configured: false,
        width: 0,
        height: 0,
        text_renderer,
        cursor_x: 0.0,
        cursor_y: 0.0,
        input_events: Vec::new(),
        // Popup state
        popup_surface: None,
        popup_layer: None,
        popup_pool: None,
        popup_configured: false,
        popup_width: 0,
        popup_height: 0,
        popup_redraw_requested: false,
        popup_input_events: Vec::new(),
    };

    event_queue
        .roundtrip(&mut state)
        .context("Failed initial roundtrip")?;

    let output = monitor.and_then(|monitor_name| {
        state.output_state.outputs().find(|o| {
            state
                .output_state
                .info(o)
                .map(|info| info.name == Some(monitor_name.clone()))
                .unwrap_or(false)
        })
    });

    let surface = state.compositor_state.create_surface(&qh);
    let layer_surface = state.layer_shell.create_layer_surface(
        &qh,
        surface.clone(),
        Layer::Top,
        Some("hyprbar"),
        output.as_ref(),
    );

    let anchor = if anchor_bottom {
        Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT
    } else {
        Anchor::TOP | Anchor::LEFT | Anchor::RIGHT
    };
    layer_surface.set_anchor(anchor);
    layer_surface.set_size(0, height);
    layer_surface.set_exclusive_zone(height as i32);
    surface.commit();

    state.surface = Some(surface);

    Ok((state, event_queue, layer_surface))
}

use smithay_client_toolkit::reexports::client::QueueHandle;

/// Creates a popup surface for displaying widget popups.
/// Position is relative to screen, typically calculated from widget position.
pub fn create_popup_surface(
    state: &mut WaylandState,
    qh: &QueueHandle<WaylandState>,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    anchor_bottom: bool,
) -> Result<()> {
    // Destroy existing popup if any
    destroy_popup_surface(state);

    // Create popup pool
    let popup_pool = SlotPool::new((width * height * 4) as usize, &state.shm)
        .context("Failed to create popup pool")?;

    // Create popup surface
    let popup_wl_surface = state.compositor_state.create_surface(qh);

    // Create layer surface for popup (Overlay layer = above everything)
    let popup_layer = state.layer_shell.create_layer_surface(
        qh,
        popup_wl_surface.clone(),
        Layer::Overlay,
        Some("hyprbar-popup"),
        None, // Same output as main surface
    );

    // Configure popup anchoring and margins
    let anchor = if anchor_bottom {
        Anchor::BOTTOM | Anchor::LEFT
    } else {
        Anchor::TOP | Anchor::LEFT
    };
    popup_layer.set_anchor(anchor);
    popup_layer.set_size(width, height);
    popup_layer.set_exclusive_zone(0); // Don't reserve space
    popup_layer.set_margin(y, 0, 0, x); // top, right, bottom, left

    popup_wl_surface.commit();

    state.popup_surface = Some(popup_wl_surface);
    state.popup_layer = Some(popup_layer);
    state.popup_pool = Some(popup_pool);
    state.popup_width = width;
    state.popup_height = height;
    state.popup_configured = false;
    state.popup_redraw_requested = true;

    hyprlog::internal::debug(
        "POPUP",
        &format!("Created popup {}x{} at ({}, {})", width, height, x, y),
    );

    Ok(())
}

/// Destroys the current popup surface if it exists.
pub fn destroy_popup_surface(state: &mut WaylandState) {
    if state.popup_layer.is_some() {
        hyprlog::internal::debug("POPUP", "Destroying popup surface");
    }

    // Drop layer surface first (this also destroys the underlying protocol object)
    state.popup_layer = None;

    // Then destroy the wl_surface
    if let Some(surface) = state.popup_surface.take() {
        surface.destroy();
    }

    state.popup_pool = None;
    state.popup_configured = false;
    state.popup_width = 0;
    state.popup_height = 0;
    state.popup_redraw_requested = false;
    state.popup_input_events.clear();
}
