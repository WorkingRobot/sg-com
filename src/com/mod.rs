mod bindings;
mod context;
mod error;
mod player;

pub use bindings::{SG_SampleRate, SG_SampleType};
pub use context::SGContext;
pub use player::Player;

#[inline]
pub fn context() -> error::Result<&'static SGContext> {
    context::get()
}
