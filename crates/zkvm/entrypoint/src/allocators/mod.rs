//! Allocators for the SP1 zkVM.
//!
//! The `embedded` allocator takes precedence if enabled. A static library build (`staticlib`)
//! allocates from a private buffer instead.

#[cfg(feature = "bump")]
mod bump;

#[cfg(all(not(feature = "bump"), not(feature = "staticlib")))]
pub mod embedded;

#[cfg(all(not(feature = "bump"), not(feature = "staticlib")))]
pub use embedded::init;

#[cfg(all(not(feature = "bump"), feature = "staticlib"))]
pub mod staticlib;

#[cfg(all(not(feature = "bump"), feature = "staticlib"))]
pub use staticlib::init;
