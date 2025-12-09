pub mod blitter;
pub mod handlers;
pub mod state;

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

pub fn init() -> Result<(WaylandState, EventQueue<WaylandState>, LayerSurface)> {
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
    };

    // Create Layer Surface
    let surface = state.compositor_state.create_surface(&qh);
    let layer_surface = state.layer_shell.create_layer_surface(
        &qh,
        surface.clone(),
        Layer::Top,
        Some("kitchnsink"),
        None,
    );

    // Initial configuration
    layer_surface.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer_surface.set_size(0, 30); // Default height 30, width max
    layer_surface.set_exclusive_zone(30);
    surface.commit(); // Changed from layer_surface.commit()

    // Store WlSurface in state for reference if needed
    state.surface = Some(surface);

    // Roundtrip to process initial events (requires mutable state)
    event_queue
        .roundtrip(&mut state)
        .context("Failed initial roundtrip")?;

    Ok((state, event_queue, layer_surface))
}
