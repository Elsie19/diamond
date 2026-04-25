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
//! * conversions
//!     - [`itoa`](itoa::itoa)
//!     - [`atoi`](itoa::atoi)
//! * numbers
//!     - [`max`](math::max)
//!     - [`min`](math::min)
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

/// Basic math.
pub mod math;
