use vstd::prelude::*;
use vstd::prelude::Ghost as Ghost;

// struct Ghost 只是注释里的举例
/* type Tracked = u8; 块注释里的举例 */
pub const NOTE: &str = "struct Ghost 写在字符串里";
pub const QUOTE: char = '"';
pub struct GhostCount(u64);
