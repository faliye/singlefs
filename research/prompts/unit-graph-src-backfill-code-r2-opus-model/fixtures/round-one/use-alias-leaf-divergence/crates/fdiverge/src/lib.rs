pub mod first {
    pub fn old_total() -> u32 {
        1
    }

    pub fn helper_total() -> u32 {
        2
    }
}

pub mod second {
    pub fn helper_total() -> u32 {
        3
    }
}

use crate::first::old_total as helper_total;

pub fn entry_point() -> u32 {
    helper_total()
}
