//! Functions to render different parts of the UI.

mod accounts;
mod guide;
mod log;
mod missing;
mod tabs;

pub use self::log::log;
pub use accounts::accounts;
pub use guide::guide;
pub use missing::missing;
pub use tabs::tabs;
pub use tabs::MenuItem;

/// Modular arithmetic with a given modulo, current value, step size, and direction.
pub fn step(modulo: usize, n: usize, size: usize, positive: bool) -> usize {
    match positive {
        true => (n + size) % modulo,
        false => (n + modulo - size) % modulo,
    }
}

/// Shorthand for moving to the immediately next step
pub fn step_next(modulo: usize, n: usize) -> usize {
    step(modulo, n, 1, true)
}

/// Shorthand for moving to the immediately previous step
pub fn step_prev(modulo: usize, n: usize) -> usize {
    step(modulo, n, 1, false)
}
