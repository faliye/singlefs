use crate::holder::Holder as Tracked;
use crate::{wrap::Wrapper as Ghost, other::Plain};

mod holder { pub struct Holder; }
mod wrap { pub struct Wrapper; }
mod other { pub struct Plain; }
