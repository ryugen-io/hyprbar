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

    // In SCTK 0.17+, RegistryState::new returns the state. EventQueue is created separately.
    let (globals, mut event_queue) =
        registry_queue_init::<WaylandState>(&conn).context("Failed to init registry queue")?;
    let qh = event_queue.handle();

    // Registry state
    let registry_state = RegistryState::new(&globals);

    // Initialize SCTK states using globals
    let compositor_state =
        CompositorState::bind(&globals, &qh).context("Failed to bind compositor")?;
    let layer_shell = LayerShell::bind(&globals, &qh).context("Failed to bind layer shell")?;
    let shm = Shm::bind(&globals, &qh).context("Failed to bind shm")?;
    let output_state = OutputState::new(&globals, &qh);
    let seat_state = SeatState::new(&globals, &qh);

    // Init SlotPool
    let pool = SlotPool::new(1920 * 1080 * 4, &shm).context("Failed to create Shm pool")?;

    // Init TextRenderer
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
    };

    // Roundtrip to populate globals (especially outputs)
    // We need to know about outputs BEFORE creating the layer surface if we want to bind to a specific one
    event_queue
        .roundtrip(&mut state)
        .context("Failed initial roundtrip")?;

    // Find the requested output
    let output = if let Some(monitor_name) = monitor {
        if monitor_name == "primary" {
            None // Use compositor default / primary behavior (often first output, or null)
        // NOTE: LayerShell spec says if output is NULL, it maps to the "default output" (usually focused).
        // However, "primary" in config usually implies the "primary monitor" which Wayland doesn't strictly have concept for in core protocol.
        // Often users just mean "default". If they want a specific one, they use the name.
        } else {
            state.output_state.outputs().find(|o| {
                state
                    .output_state
                    .info(o)
                    .map(|info| info.name == Some(monitor_name.clone()))
                    .unwrap_or(false)
            })
        }
    } else {
        None
    };

    // Create Layer Surface
    let surface = state.compositor_state.create_surface(&qh);
    let layer_surface = state.layer_shell.create_layer_surface(
        &qh,
        surface.clone(),
        Layer::Top,
        Some("kitchnsink"),
        output.as_ref(),
    );

    // Initial configuration
    let anchor = if anchor_bottom {
        Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT
    } else {
        Anchor::TOP | Anchor::LEFT | Anchor::RIGHT
    };
    layer_surface.set_anchor(anchor);
    layer_surface.set_size(0, height);
    layer_surface.set_exclusive_zone(height as i32);
    surface.commit(); // Changed from layer_surface.commit()

    // Store WlSurface in state for reference if needed
    state.surface = Some(surface);

    // Roundtrip to process initial events (requires mutable state)
    // Roundtrip to process initial events (requires mutable state)
    // event_queue
    //    .roundtrip(&mut state)
    //    .context("Failed initial roundtrip")?;

    Ok((state, event_queue, layer_surface))
}
