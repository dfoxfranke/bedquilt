//! Error types.

/// The error type for all Bedquilt IO functions.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Error {
    /// Glk returned an error value.
    /// 
    /// No further information is available programatically, but the
    /// Glk implementation may have printed something to the screen.
    GlkError,
    /// The Glk area is full.
    /// 
    /// This error can occur on Glulx if you attempt to request line input from
    /// more than the supported number of windows at one time. It should never
    /// occur on other platforms.
    GlkAreaAllocError,
    /// The Glk implementation does not support the requested operation.
    Unsupported
}

/// The result type for all Bedquilt IO functions.
pub type Result<T> = core::result::Result<T, Error>;
