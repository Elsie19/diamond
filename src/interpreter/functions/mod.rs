//! Here is the canonical source for Diamond's standard library.
//!
//! * arrays
//!     - [`nth`](arrays::nth)
//!     - [`split`](arrays::split)
//!     - [`len`](arrays::len)
//!     - [`enumerate`](arrays::enumerate)
//!     - [`last`](arrays::last)
//!     - [`chars`](arrays::chars)
//! * files
//!     - [`file`](file::file)
//!     - [`create`](file::create)
//!     - [`open`](file::open)
//!     - [`dump`](file::dump)
//!     - [`lines`](file::lines)
//!     - [`skip`](file::skip)
//! * printf and friends
//!     - [`printf`](printf::printf)
//!     - [`sprintf`](printf::sprintf)
//!     - [`puts`](printf::puts)
//! * system
//!     - [`args`](system::args)
//!     - [`exit`](system::exit)
//!     - [`panic`](system::panic)

/// Array functions.
pub mod arrays;

/// Debugging tools.
pub mod dump_var;

/// File and stream related functions.
pub mod file;

/// Conversions.
pub mod itoa;

/// Printing to stdout.
pub mod printf;

/// System related stuff.
pub mod system;

pub mod testing_branch;
