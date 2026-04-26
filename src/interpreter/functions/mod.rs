//! Here is the canonical source for Diamond's standard library.
//!
//! * arrays
//!     - [`nth`](arrays::nth)
//!     - [`split`](arrays::split)
//!     - [`len`](arrays::len)
//!     - [`enumerate`](arrays::enumerate)
//!     - [`last`](arrays::last)
//!     - [`chars`](arrays::chars)
//!     - [`only`](arrays::only)
//! * files
//!     - [`file`](file::file)
//!     - [`create`](file::create)
//!     - [`open`](file::open)
//!     - [`dump`](file::dump)
//!     - [`lines`](file::lines)
//!     - [`skip`](file::skip)
//!     - [`fpop`](file::fpop)
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
//!     - [`add`](math::add)
//!     - [`sub`](math::sub)
//! * system
//!     - [`args`](system::args)
//!     - [`exit`](system::exit)
//!     - [`panic`](system::panic)
//! * result
//!     - [`ok`](result::ok)
//!     - [`err`](result::err)

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

/// Result type.
pub mod result;

/// Branching tool for results.
#[doc(hidden)]
#[macro_export]
macro_rules! res {
    ($kind:ident, str => $s:literal) => {
        $crate::interpreter::types::IResultBranch::$kind(Box::new(
            $crate::interpreter::types::ILitType::String(::std::rc::Rc::from($s)),
        ))
    };
    ($kind:ident, str_dy => $s:expr) => {
        $crate::interpreter::types::IResultBranch::$kind(Box::new(
            $crate::interpreter::types::ILitType::String($s.into()),
        ))
    };
    ($kind:ident, int => $i:literal) => {
        $crate::interpreter::types::IResultBranch::$kind(Box::new(
            $crate::interpreter::types::ILitType::Integer($i),
        ))
    };
    ($kind:ident, int_dy => $i:expr) => {
        $crate::interpreter::types::IResultBranch::$kind(Box::new(
            $crate::interpreter::types::ILitType::Integer($i),
        ))
    };
    ($kind:ident, file => $i:expr) => {
        $crate::interpreter::types::IResultBranch::$kind(Box::new(
            $crate::interpreter::types::ILitType::File($i),
        ))
    };
    ($kind:ident, arr => $i:expr) => {
        $crate::interpreter::types::IResultBranch::$kind(Box::new(
            $crate::interpreter::types::ILitType::Array($i.into()),
        ))
    };
    ($kind:ident, stream => $i:expr) => {
        $crate::interpreter::types::IResultBranch::$kind(Box::new(
            $crate::interpreter::types::ILitType::Stream(
                $crate::interpreter::types::IStreamHandle::File(::std::rc::Rc::new(
                    ::std::cell::RefCell::new($i),
                )),
            ),
        ))
    };
    ($kind:ident, any => $i:expr) => {
        $crate::interpreter::types::IResultBranch::$kind(Box::new($i))
    };
    ($kind:ident, unit) => {
        $crate::interpreter::types::IResultBranch::$kind(Box::new(
            $crate::interpreter::types::ILitType::Unit,
        ))
    };
}
