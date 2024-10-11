#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GlkError;

pub type Result<T> = core::result::Result<T, GlkError>;
