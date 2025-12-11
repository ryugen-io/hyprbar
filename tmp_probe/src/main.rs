use ks_core::state::BarState;

fn main() {
    let state: BarState = unsafe { std::mem::zeroed() };
    // Trigger error to see available fields or confirm existence
    let _ = state.cookbook.icons; 
}
