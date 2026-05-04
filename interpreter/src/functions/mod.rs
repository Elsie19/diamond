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
//!     - [`rev`](arrays::rev)
//!     - [`skip`](arrays::skip)
//!     - [`range`](arrays::range)
//! * files
//!     - [`file`](file::file)
//!     - [`create`](file::create)
//!     - [`open`](file::open)
//!     - [`dump`](file::dump)
//!     - [`lines`](file::lines)
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
//!     - [`eq`](result::eq)
//! * strings
//!     - [`trim`](strings::trim)
//!     - [`trim_left`](strings::trim_left)
//!     - [`trim_right`](strings::trim_right)
//!     - [`upper`](strings::upper)
//!     - [`lower`](strings::lower)
//!     - [`replace`](strings::replace)
//!     - [`split_at`](strings::split_at)
//!     - [`join`](strings::join)
//!     - [`pattern_pos`](strings::pattern_pos)

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

/// Strings.
pub mod strings;

/// Branching tool for results.
#[doc(hidden)]
#[macro_export]
macro_rules! res {
    ($kind:ident, str => $s:literal) => {
        $crate::types::IResultBranch::$kind(Box::new($crate::types::ILitType::String(
            ::std::rc::Rc::from($s),
        )))
    };
    ($kind:ident, str_dy => $s:expr) => {
        $crate::types::IResultBranch::$kind(Box::new($crate::types::ILitType::String($s.into())))
    };
    ($kind:ident, int => $i:literal) => {
        $crate::types::IResultBranch::$kind(Box::new($crate::types::ILitType::Integer($i)))
    };
    ($kind:ident, int_dy => $i:expr) => {
        $crate::types::IResultBranch::$kind(Box::new($crate::types::ILitType::Integer($i)))
    };
    ($kind:ident, file => $i:expr) => {
        $crate::types::IResultBranch::$kind(Box::new($crate::types::ILitType::File($i)))
    };
    ($kind:ident, arr => $i:expr) => {
        $crate::types::IResultBranch::$kind(Box::new($crate::types::ILitType::Array($i.into())))
    };
    ($kind:ident, arr_alr_rc => $i:expr) => {
        $crate::types::IResultBranch::$kind(Box::new($crate::types::ILitType::Array($i)))
    };
    ($kind:ident, stream => $i:expr) => {
        $crate::types::IResultBranch::$kind(Box::new($crate::types::ILitType::Stream(
            ::std::rc::Rc::new(::std::cell::RefCell::new($i)),
        )))
    };
    ($kind:ident, any => $i:expr) => {
        $crate::types::IResultBranch::$kind(Box::new($i))
    };
    ($kind:ident, unit) => {
        $crate::types::IResultBranch::$kind(Box::new($crate::types::ILitType::Unit))
    };
}
