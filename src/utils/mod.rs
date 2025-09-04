// Feature-dependent modules
#[cfg(feature = "self_update")]
mod update;
#[cfg(feature = "self_update")]
pub use update::{check_for_updates, update};
