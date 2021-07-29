#[cfg(feature = "with-tui")]
mod tui_impl;
#[cfg(feature = "with-tui")]
pub use tui_impl::render;

#[cfg(feature = "with-flat")]
mod flat_impl;
#[cfg(feature = "with-flat")]
pub use flat_impl::render;
