//! Here is the canonical source for Diamond's standard library.
//!
//! * arrays
//!     - [`nth`](arrays::nth)
//!     - [`split`](arrays::split)
//!     - [`len`](arrays::len)
//!     - [`enumerate`](arrays::enumerate)
//!     - [`last`](arrays::last)
//! * files
//!     - [`file`](file::file)
//!     - [`create`](file::create)
//!     - [`open`](file::open)
//!     - [`dump`](file::dump)
//!     - [`lines`](file::lines)
//! * printf and friends
//!     - [`printf`](printf::printf)
//!     - [`sprintf`](printf::sprintf)
//!     - [`puts`](printf::puts)

/// Array functions.
pub mod arrays;

/// Debugging tools.
pub mod dump_var;

/// File and stream related functions.
pub mod file;

/// Conversions.
pub mod itoa;

/// System utilities.
pub mod panic;

/// Printing to stdout.
pub mod printf;

pub mod testing_branch;
